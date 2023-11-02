import testing2

result = testing2.sum_as_string(10, 11)

print(result)

graph = testing2.Graph()

class SubGraph(testing2.Graph):
    def __init__(self):
        print("Hello")
        pass

subgraph = SubGraph()

subgraph.add_node("Lmao")
subgraph.add_node("Lmao")

print(f"Bruv {subgraph}")