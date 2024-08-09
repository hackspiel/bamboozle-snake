import random
import time
from collections import defaultdict, deque
from random import shuffle
from typing import List
from copy import deepcopy

from tqdm import tqdm

from game import Game
from snake import Snake
from heuristic import Heuristic


class Population:
    def __init__(self, snakes: List[Snake]):
        self.snakes = snakes


    def update_snakes(self, sorted_wins):
        # first two snakes will be kept (or more if they have the same number of wins)
        wins_2nd = sorted_wins[list(sorted_wins.keys())[1]]
        new_snakes = [deepcopy(k) for k in sorted_wins.keys() if sorted_wins[k] >= wins_2nd]

        # add mutations of the first two snakes
        for i in range(len(new_snakes)):
            new_snakes.append(new_snakes[i].mutate())

        # add mutations of random snakes
        for new_snake in random.choices(list(sorted_wins.keys()), weights=list(sorted_wins.values()),
                                        k=max(1, len(self.snakes) - len(new_snakes) - 1)):
            new_snakes.append(new_snake.mutate())

        # add one complete new snake
        new_snakes.append(Snake("", Heuristic.create_random()))

        new_snakes = new_snakes[:len(self.snakes)]

        for i in range(len(new_snakes)):
            new_snakes[i].name = f"snake{i}"

        self.snakes = new_snakes

    def step(self):
        wins = self.eval()
        sorted_winners = sorted(wins, key=wins.get, reverse=True)
        name_to_snake = {snake.name: snake for snake in self.snakes}
        sorted_wins = {name_to_snake[k]: wins[k] for k in sorted_winners}

        self.update_snakes(sorted_wins)
        
        return sorted_wins

    def eval(self, max_parallel_games: int = 7):
        wins = defaultdict(int)

        for i, snake in enumerate(self.snakes):
            snake.start_server(8080 + i, timeout=400)
            # print(snake.process.stderr.read())
        


        games = []
        for i in range(len(self.snakes)):
            for j in range(i + 1, len(self.snakes)):
                games.append(Game(snakes=[self.snakes[i], self.snakes[j]], sequential=True, browser=False))
                games.append(Game(snakes=[self.snakes[i], self.snakes[j]], sequential=True, browser=False))

        with tqdm(total=len(games)) as pbar:
            shuffle(games)
            games_queue = deque(games)
            running_games = []
            finished_games = []
            while len(running_games) > 0 or len(games_queue) > 0:
                while len(running_games) < max_parallel_games and len(games_queue) > 0:
                    game = games_queue.pop()
                    game.start()
                    running_games.append(game)
                time.sleep(0.5)
                # print(f"Running games: {len(running_games)}")
                for i in reversed(range(len(running_games))):
                    if not running_games[i].is_alive():
                        game = running_games.pop(i)
                        finished_games.append(game)
                        if game.winner == "DRAW":
                            wins[game.snakes[0].name] += 0.5
                            wins[game.snakes[1].name] += 0.5
                        else:
                            wins[game.winner] += 1
                        pbar.update(1)

        for snake in self.snakes:
            snake.stop_server()

        return wins
