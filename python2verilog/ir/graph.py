"""
Graph representation of the logic

Naming convention:
Vertex
Edge
Element := Vertex | Edge
"""

from __future__ import annotations

from typing import Iterator, Optional

from python2verilog.ir import expressions as expr
from python2verilog.utils.typed import guard, typed, typed_strict


def get_variables(exp: expr.Expression) -> Iterator[expr.Var]:
    """
    Gets variables from expression
    """
    if isinstance(exp, expr.UBinOp):
        yield from get_variables(exp.left)
        yield from get_variables(exp.right)
    elif isinstance(exp, expr.UnaryOp):
        yield from get_variables(exp.expr)
    elif isinstance(exp, expr.Var):
        yield exp
    elif isinstance(exp, (expr.UInt, expr.Int)):
        pass
    else:
        raise RuntimeError(f"{type(exp)}")


class Element:
    """
    Element, base class for vertex or edge
    """

    def __init__(self, unique_id: str, name: str = ""):
        self.name = typed_strict(name, str)
        self.unique_id = typed_strict(unique_id, str)

    def __hash__(self) -> int:
        return hash(self.unique_id)

    def __eq__(self, __value: object):
        if isinstance(__value, Element):
            return self.unique_id == __value.unique_id
        return False

    def visit_nonclocked(self) -> Iterator[Element]:
        """
        Yields self and recursively yields optimal nonclocked children of element

        :return: [children_branch_0, children_branch_1, ...]
        """
        yield from ()

    def view_children(self) -> str:
        """
        Views children of node
        """
        return str(list(self.visit_nonclocked()))

    def children(self) -> Iterator[Element]:
        """
        Gets children of node
        """
        yield from ()

    def exclusions(self) -> Iterator[str]:
        """
        Yields all exclusion groups that will be read or written to
        within this group of nonclocked nodes.

        The reason reads are also included is because checking a callee's
        ready signal more than once in a clock cycle is usually incorrect.
        """
        yield from ()


class BasicElement(Element):
    """
    Basic element with a single child
    """

    def __init__(
        self,
        unique_id: str,
        *args,
        child: Optional[Element] = None,
        **kwargs,
    ):
        super().__init__(unique_id, *args, **kwargs)
        self._child = typed(child, Element)
        self._optimal_child: Optional[Element] = None

    def visit_nonclocked(self) -> Iterator[Element]:
        if isinstance(self, ClockedEdge):
            yield self
            yield self.optimal_child
        elif self.optimal_child:
            yield self
            yield from self.optimal_child.visit_nonclocked()

    @property
    def child(self) -> Element:
        """
        child or optimal_child if no child
        """
        assert self._child, f"{self} {self.view_children()}"
        return typed_strict(self._child, Element)

    @child.setter
    def child(self, other: Element):
        self._child = typed_strict(other, Element)

    def has_child(self) -> bool:
        """
        Returns True if has child
        """
        return self._child is not None

    def children(self):
        if self._child:
            yield self._child
        if self._optimal_child:
            yield self._optimal_child

    @property
    def optimal_child(self):
        """
        Optimal child or child otherwise
        """
        return self._optimal_child if self._optimal_child else self._child

    @optimal_child.setter
    def optimal_child(self, other: Element):
        self._optimal_child = typed_strict(other, Element)


class Node(Element):
    """
    Vertex
    """

    def __repr__(self) -> str:
        return self.name


class IfElseNode(Node, Element):
    """
    Represents an if-else statement
    """

    def __init__(
        self,
        unique_id: str,
        *args,
        true_edge: Edge,
        false_edge: Edge,
        condition: expr.Expression,
        **kwargs,
    ):
        super().__init__(unique_id, *args, **kwargs)
        self.true_edge = typed_strict(true_edge, Edge)
        self.false_edge = typed_strict(false_edge, Edge)
        self.condition = typed_strict(condition, expr.Expression)
        self._optimal_true_edge: Optional[Edge] = None
        self._optimal_false_edge: Optional[Edge] = None

    @property
    def optimal_true_edge(self):
        """
        optimal true edge or edge otherwise
        """
        return self._optimal_true_edge if self._optimal_true_edge else self.true_edge

    @optimal_true_edge.setter
    def optimal_true_edge(self, other: Edge):
        self._optimal_true_edge = typed_strict(other, Edge)

    @property
    def optimal_false_edge(self):
        """
        optimal false edge
        """
        return self._optimal_false_edge if self._optimal_false_edge else self.false_edge

    @optimal_false_edge.setter
    def optimal_false_edge(self, other: Edge):
        self._optimal_false_edge = typed_strict(other, Edge)

    def children(self) -> Iterator[Edge]:
        """
        Gets edges
        """
        if self.true_edge:
            yield self.true_edge
        if self.false_edge:
            yield self.false_edge
        if self._optimal_true_edge:
            yield self._optimal_true_edge
        if self._optimal_false_edge:
            yield self._optimal_false_edge

    def exclusions(self):
        for var in get_variables(self.condition):
            if isinstance(var, expr.ExclusiveVar):
                yield var.exclusive_group
        yield from self.optimal_true_edge.exclusions()
        yield from self.optimal_false_edge.exclusions()

    def __repr__(self):
        return f"If{self.condition}"

    def visit_nonclocked(self) -> Iterator[Element]:
        yield self
        yield Node(unique_id="", name="True Branch")
        yield from self.optimal_true_edge.visit_nonclocked()
        yield Node(unique_id="", name="False Branch")
        yield from self.optimal_false_edge.visit_nonclocked()


class BasicNode(Node, BasicElement):
    """
    Basic node.
    Has one child.
    """

    def __init__(self, unique_id: str, *args, child: Edge | None = None, **kwargs):
        super().__init__(unique_id, *args, **kwargs)
        self._child = child

    @property
    def edge(self) -> Edge:
        """
        Gets edge
        """
        assert guard(self._child, Edge)
        return self._child

    @edge.setter
    def edge(self, other: Edge):
        assert guard(other, Edge)
        self._child = other


class AssignNode(BasicNode):
    """
    Represents a non-blocking assignment,
    i.e. assignments that do not block the execution of
    the next statements, without a clock cycle having to pass
    """

    def __init__(
        self,
        unique_id: str,
        *args,
        lvalue: expr.Var,
        rvalue: expr.Expression,
        child: Optional[Edge] = None,
        **kwargs,
    ):
        super().__init__(unique_id, *args, child=child, **kwargs)
        self.lvalue = typed_strict(lvalue, expr.Var)
        self.rvalue = typed_strict(rvalue, expr.Expression)
        self._child = typed(child, Edge)

    def __repr__(self):
        return f"{self.lvalue} = {self.rvalue}"

    def exclusions(self):
        for var in get_variables(self.lvalue):
            if isinstance(var, expr.ExclusiveVar):
                yield var.exclusive_group
        yield from get_variables(self.rvalue)
        if self._child:
            yield from self.child.exclusions()


class Edge(BasicElement):
    """
    Represents an edge between two vertices
    """

    def __init__(self, unique_id: str, *args, child: Element | None = None, **kwargs):
        super().__init__(unique_id, *args, child=child, **kwargs)
        self._child = child

    def get_name(self):
        """
        Gets edge name
        """
        return self.name


class NonClockedEdge(Edge):
    """
    Represents a non-clocked edge,
    i.e. no clock cycle has to pass for the next node to be executed
    """

    def exclusions(self):
        yield from self.child.exclusions()

    def __repr__(self) -> str:
        return "=>"


class ClockedEdge(Edge):
    """
    Represents a clocked edge,
    i.e. a clock cycle has to pass for the next node to be executed
    """

    def exclusions(self):
        yield from ()

    def __repr__(self):
        return "=/>"


def create_networkx_adjacency_list(node: Element):
    """
    Creates adjacency list from a node

    Assumes names are unique
    """
    adjacency_list = {}

    def traverse_graph(curr_node: Element, visited: set[Element]):
        if not curr_node:
            return

        nonlocal adjacency_list
        if curr_node in visited:
            return

        visited.add(curr_node)
        children = curr_node.children()
        adjacency_list[curr_node] = children

        for child in children:
            traverse_graph(child, visited)

    traverse_graph(node, set())
    return adjacency_list


def create_cytoscape_elements(node: Element):
    """
    Creates adjacency list from a node

    Assumes names are unique
    """
    nodes = []
    edges = []

    def traverse_graph(curr_node: Element, visited: set[str]):
        if not curr_node:
            return

        nonlocal nodes
        if curr_node.unique_id in visited:
            return

        visited.add(curr_node.unique_id)
        children = curr_node.children()

        if not isinstance(curr_node, Edge):
            nodes.append(
                {
                    "data": {
                        "id": curr_node.unique_id,
                        "label": str(curr_node),
                        "class": str(curr_node.__class__.__name__),
                    }
                }
            )
            for child in curr_node.children():
                assert guard(child, BasicElement)
                edges.append(
                    {
                        "data": {
                            "source": curr_node.unique_id,
                            "target": child.child.unique_id,
                            "class": str(child.__class__.__name__),
                            "label": str(child),
                        }
                    }
                )

            for child in children:
                assert guard(child, BasicElement)
                traverse_graph(child.child, visited)

    traverse_graph(node, set())
    return {"nodes": nodes, "edges": edges}
