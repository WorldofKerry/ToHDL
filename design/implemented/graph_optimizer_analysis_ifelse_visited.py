# 1 return states
(
    "state _1_while",
    "if ((i < n)) then",
    "if (i > 1) then",
    "i <= (i + 1)",
    "goto _1_while",
)
(
    "state _2",
    "i <= 0",
    "if ((0 < n)) then",
    "if (0 > 1) then",
    "i <= (0 + 1)",
    "goto _1_while",
)
(
    "state _1_while",
    "if ((i < n)) then",
    "if (i > 1) else",
    "i <= (i + 2)",
    "goto _1_while",
)
("state _2", "i <= 0", "if ((0 < n)) else", "out0 <= 0", "goto done")
(
    "state _2",
    "i <= 0",
    "if ((0 < n)) then",
    "if (0 > 1) else",
    "i <= (0 + 2)",
    "goto _1_while",
)
("state _1_while", "if ((i < n)) else", "out0 <= i", "goto done")

# 2 states update and return states
("state _1_while", "goto _1_edge_o19 _1_f_o32")
(
    "state _1_while",
    "if ((i < n)) then",
    "if (i > 1) else",
    "i <= (i + 2)",
    "goto _1_while",
)
(
    "state _1_while",
    "if ((i < n)) then",
    "if (i > 1) then",
    "i <= (i + 1)",
    "goto _1_while",
)
("state _1_while", "if ((i < n)) else", "out0 <= i", "goto done")
(
    "state _2",
    "i <= 0",
    "if ((0 < n)) then",
    "if (0 > 1) else",
    "i <= (0 + 2)",
    "goto _1_while",
)
("state _2", "i <= 0", "if ((0 < n)) else", "out0 <= 0", "goto done")
(
    "state _2",
    "i <= 0",
    "if ((0 < n)) then",
    "if (0 > 1) then",
    "i <= (0 + 1)",
    "goto _1_while",
)

# 3 pass
(
    "state _1_while",
    "if ((i < n)) then",
    "if (i > 1) then",
    "i <= (i + 1)",
    "goto _1_while",
)
("state _1_while", "if ((i < n)) else", "out0 <= i", "goto done")
(
    "state _2",
    "i <= 0",
    "if ((0 < n)) then",
    "if (0 > 1) then",
    "i <= (0 + 1)",
    "goto _1_while",
)
("state _1_while", "if ((i < n)) then", "goto _1_while_0_o14")
("state _2", "i <= 0", "if ((0 < n)) else", "out0 <= 0", "goto done")
("state _1_while", "if ((i < n)) else", "goto _0_o31")
(
    "state _1_while",
    "if ((i < n)) then",
    "if (i > 1) else",
    "i <= (i + 2)",
    "goto _1_while",
)
(
    "state _2",
    "i <= 0",
    "if ((0 < n)) then",
    "if (0 > 1) else",
    "i <= (0 + 2)",
    "goto _1_while",
)
# set difference between 3 and 2
("state _1_while", "if ((i < n)) then", "goto _1_while_0_o14")
("state _1_while", "if ((i < n)) else", "goto _0_o31")

# conclusion: option 1 seems to be sufficiently complete

# used python code:
def testing(n) -> tuple[int]:
    i = 0
    while i < n:
        if i > 1:
            i += 1
        else:
            i += 2
    yield (i,)
