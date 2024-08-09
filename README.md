### Bamboozle Snake
An AI agent for the game [BattleSnake](https://play.battlesnake.com/) that utilizes a paranoid Alpha-Beta search algorithm.

The agent is implemented in Rust, with the core logic located in the `bamboozle_snake/` directory.

Additionally, there's an evolutionary optimization module in the `optim/` directory, which is used to fine-tune heuristic weights. Note that this requires the [official BattleSnake engine](https://github.com/BattlesnakeOfficial/rules).