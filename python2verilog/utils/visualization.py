"""
Visualization Tools
"""

import logging
from typing import Iterator, Optional, Union, cast

import matplotlib
import matplotlib.pyplot as plt  # type: ignore
import numpy as np


def make_visual(generator_inst, directory: Optional[str] = None):
    """
    Any iterable of tuples where the tuples are of length > 0 will work.
    Visualizes the first 3 elements of each tuple as (x, y, colour)
    """

    def make_triple(
        inst: Iterator[Union[tuple[int, ...], int]]
    ) -> Iterator[tuple[int, int, int]]:
        """
        Makes a generator yield 3 values by
        truncating or padding values
        """
        for idx, output in enumerate(inst):
            if isinstance(output, int):
                output = (output,)
            if max(output) > 1000 or idx > 1000:
                return  # plot will be too big
            if len(output) >= 3:
                yield cast(tuple[int, int, int], output[:3])
            elif len(output) == 2:
                yield cast(tuple[int, int, int], ((*output, 1)))
            else:
                yield cast(tuple[int, int, int], ((*output, idx, 1)))

    data = list(make_triple(generator_inst))
    data_triple = np.array(data)

    try:
        height = max(data_triple[:, 0])
        width = max(data_triple[:, 1])
        grid = np.zeros((int(height) + 1, int(width) + 1))
        for x_coord, y_coord, colour in data_triple:
            grid[x_coord, y_coord] = colour

        # Create the pixel-like plot
        plt.imshow(grid)

        # Set labels and title
        plt.xlabel("X")
        plt.ylabel("Y")
        plt.title("Colorbar Plot")

        # Add color bar
        cdict = {
            "red": (
                (0.0, 0.0, 0.0),
                (0.1, 0.5, 0.5),
                (0.2, 0.0, 0.0),
                (0.4, 0.2, 0.2),
                (0.6, 0.0, 0.0),
                (0.8, 1.0, 1.0),
                (1.0, 1.0, 1.0),
            ),
            "green": (
                (0.0, 0.0, 0.0),
                (0.1, 0.0, 0.0),
                (0.2, 0.0, 0.0),
                (0.4, 1.0, 1.0),
                (0.6, 1.0, 1.0),
                (0.8, 1.0, 1.0),
                (1.0, 0.0, 0.0),
            ),
            "blue": (
                (0.0, 0.0, 0.0),
                (0.1, 0.5, 0.5),
                (0.2, 1.0, 1.0),
                (0.4, 1.0, 1.0),
                (0.6, 0.0, 0.0),
                (0.8, 0.0, 0.0),
                (1.0, 0.0, 0.0),
            ),
        }

        plt.pcolor(
            grid,
            cmap=matplotlib.colors.LinearSegmentedColormap(
                "my_colormap", cdict, 256  # type: ignore
            ),
        )
        cbar = plt.colorbar()
        cbar.set_label("Z")

        plt.gca().invert_yaxis()

        # Show the plot
        plt.show()
        if directory:
            plt.savefig(directory)

        plt.clf()
        plt.cla()
        plt.close()
    except IndexError as e:
        logging.warning(
            "Skipping make_visual for %s due to negative outputs %s", data, e
        )

    return data
