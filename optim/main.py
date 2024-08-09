from copy import deepcopy

from snake import Snake
from game import Game
from heuristic import Heuristic
from population import Population
import pickle

if __name__ == "__main__":
    default_heuristic = Heuristic(
        {"area": 1.0, "snake_area": 0.1,"hazard_area":0.1, "health": 0.05,
         "length": 0.0, "food": 0.0})

    last_best = Heuristic(
        {"area": 1.0, "snake_area": 0.1,"hazard_area":0.1, "health": 0.05,
         "length": 0.0, "food": 0.0})

    start_snakes = [Snake("snake0", default_heuristic), Snake("snake1", last_best)]

    for i in range(8):
        start_snakes.append(Snake(f"snake{i + 1}", Heuristic.create_random()))

    population = Population(start_snakes)

    populations = []

    for i in range(100):
        step_res = population.step()
        pop = {k.heuristic: v for k, v in step_res.items()}
        print(f"Step {i + 1}")
        for k, v in pop.items():
            print(f"{v}: {k}")
        populations.append(pop)
        pickle.dump(populations, open("generations.pkl", "wb"))
