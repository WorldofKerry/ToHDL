from testing2 import Node, Graph


class SubGraph(Graph):
    def __init__(self):
        print("Hello")
        pass

class BranchNode(Node):
    def __init__(self, name):
        super().__init__()
        pass

    def __repr__(self):
        return f"BranchNode({self.name})"

    def __str__(self):
        return f"BranchNode({self.name})"

subgraph = SubGraph()

prev = subgraph.add_node(BranchNode("prev"))
next = subgraph.add_node(BranchNode("next"))

print(f"Bruv {subgraph}")
print(f"{prev=}")
