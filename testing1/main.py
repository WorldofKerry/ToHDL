from testing2 import Graph
import timeit


class Node2:
    """
    Element, base class for vertex or edge
    """

    def __init__(self, unique_id: str = ""):
        self._unique_id = unique_id

    def __hash__(self) -> int:
        assert len(self.unique_id) > 0, f'"{self.unique_id}"'
        return hash(self.unique_id)

    def __eq__(self, value: object):
        if isinstance(value, Node2):
            return self.unique_id == value.unique_id
        return False

    @property
    def unique_id(self):
        """
        Unique id
        """
        return self._unique_id

    def set_unique_id(self, unique_id: str):
        """
        Sets unique id
        """
        assert len(self._unique_id) == 0
        self._unique_id = unique_id, str

    def __repr__(self) -> str:
        return f"{self.unique_id}"


subgraph = Graph()

start = timeit.default_timer()

for _ in range(100000):
    subgraph.add_node(Node2())

end = timeit.default_timer()

print(f"Time taken: {end - start}")

# print(f"Bruv {subgraph.nodes()}")
print(f"{subgraph.to_dot()}")
