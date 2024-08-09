from copy import deepcopy

from snake import Snake
from game import Game
from heuristic import Heuristic
import pickle
from population import Population
import pickle

if __name__ == "__main__":

    generations = pickle.load(open("generations.pkl", "rb"))
    latest_heuristics = generations[-1]
    snakes = []
    sorted_wins = {}

    for i, h in enumerate(latest_heuristics.keys()):
        snake = Snake(f"snake{i}", h)
        snakes.append(snake)
        sorted_wins[snake] = latest_heuristics[h]
        print(f"{latest_heuristics[h]}: {h}")

    population = Population(snakes)
    population.update_snakes(sorted_wins)

    for i in range(len(generations), 100):
        print(f"Step {i + 1}")
        step_res = population.step()
        pop = {k.heuristic: v for k, v in step_res.items()}
        for k, v in pop.items():
            print(f"{v}: {k}")
        generations.append(pop)
        pickle.dump(generations, open("generations.pkl", "wb"))
