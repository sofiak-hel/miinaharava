## Project specification

- The project will be written in Rust.
  - I can do peer reviews in the following programming languages: Java, Python, Lua, C, C++, JavaScript, TypeScript, Go, C#
- The problem I'm solving is playing Minesweeper optimally, and I will implement an algorithm that solves games by modeling them as a constraint satisfication problem, using the Coupled Subsets CSP algorithm from David Becerra's article [0]. This algorithm analyzes sets of constraints, which are created based on the state of the game. The constraint sets will be represented by many lists of constraints, as the top-level data structure. The constraint data structure will be a tuple, consisting of the label (N) and a list of variables (A, B, C...) which will form the constraint `A + B + C ...= N`.
- The program will take inputs through a GUI, and possibly command line arguments. The GUI will show the state of the game, so the user can follow the games being played out.
- The complexity of Minesweeper is hard to analyze, because it is an NP-complete problem. The chances of winning a game can however be optimized and measured, so the aim is to reach the win-rates presented in Becerra's article [0]: 90% for beginner levels, 75% for intermediate levels, and 30% for expert levels.
  - Beginner levels are of size 8x8, 9x9 or 10x10, and they have 10 mines.
  - Intermediate levels are between size 13x15 and 16x16, and they have 40 mines.
  - Expert levels are of size 16x30 or 30x16, and they have 99 mines.
- Sources:
  - [0] <https://dash.harvard.edu/bitstream/handle/1/14398552/BECERRA-SENIORTHESIS-2015.pdf>
  - [1] <https://www.cs.toronto.edu/~cvs/minesweeper/minesweeper.pdf>
- Degree programme: tietojenkäsittelytieteen kandidaatti (TKT).
- The project documentation will use English as the language.