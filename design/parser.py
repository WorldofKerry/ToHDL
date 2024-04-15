import ast

import networkx as nx


class CFGConverter(ast.NodeVisitor):
    def __init__(self):
        self.graph = nx.DiGraph()
        self.current_node = 0
        self.loop_stack = []

    def generic_visit(self, node):
        print(f"{ast.dump(node)}")
        super().generic_visit(node)

    def visit_Assign(self, node):
        target = self.visit_Name(node.targets[0])
        print(f"{target=}")
        return

    def visit_Name(self, node):
        return node.id


# Example AST
source_code = """
def func():
    i = 0
    while 1:
        if i == 10:
            break
        if i == 5:
            continue
        i += 1
        yield i
"""

# Parse the AST
tree = ast.parse(source_code)

# Convert AST to CFG
converter = CFGConverter()
converter.visit(tree.body[0])

# Plot the graph using matplotlib
import matplotlib.pyplot as plt

pos = nx.spring_layout(converter.graph)
nx.draw(
    converter.graph,
    pos,
    with_labels=True,
    font_weight="bold",
    node_size=800,
    node_color="lightblue",
    font_size=10,
)

# Write graph to file as graphviz dot file
nx.drawing.nx_pydot.write_dot(converter.graph, "test.dot")
