from typing import Dict

import numpy as np


class Heuristic:
    def __init__(self, parameters: Dict[str, float]):
        self.params = parameters

    @staticmethod
    def create_random():
        parameters = {
            "area": np.random.random(),
            "snake_area": np.random.random(),
            "hazard_area": np.random.random(),
            "health": np.random.random(),
            "length": np.random.random(),
            "food": np.random.random(),
        }
        return Heuristic(parameters)

    def normalize(self):
        param_sum = 0
        for k, v in self.params.items():
            if k != "snake_area" or k != "hazard_area":
                param_sum += v
                
        for key in self.params:
            if key != "snake_area":
                self.params[key] /= param_sum

    def mutate(self, mutation_chance: float = 0.3, normalize: bool = True, sigma: float = 0.08):

        for key in self.params:
            if mutation_chance <= np.random.random():
                self.params[key] = np.random.normal(self.params[key], sigma)

        if normalize:
            self.normalize()

    def __str__(self):
        return str(self.params).replace("'", "\"")
