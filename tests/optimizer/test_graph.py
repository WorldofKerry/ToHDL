import unittest

import networkx as nx
from matplotlib import pyplot as plt

from python2verilog.ir import *


class TestGraph(unittest.TestCase):
    def test_visual(self):
        pass
        # e0to1 = Edge()
        # line0 = AssignNode(name="root", lvalue=Var("i"), rvalue=Int(0), edge=e0to1)
        # line1 = IfElseNode(
        #     name="while",
        #     condition=Expression("i < w"),
        #     then_edge=Edge("True"),
        #     else_edge=Edge("False"),
        # )
        # e0to1.set_next_node(line1)
        # adjacency_list = create_adjacency_list(line0)
        # g = nx.DiGraph(adjacency_list)
        # nx.draw(g, with_labels=True, font_weight="bold")

        # plt.savefig("path.png")
