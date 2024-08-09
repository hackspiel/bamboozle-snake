import os
import threading
import subprocess
from copy import deepcopy


class Snake:
    def __init__(self, name, heuristic):
        self.port = None
        self.process = None
        self.name = name
        self.heuristic = heuristic

    def mutate(self) -> "Snake":
        new_snake = deepcopy(self)
        new_snake.heuristic.mutate()
        return new_snake

    def start_server(self, port: int, timeout: int = 444):
        self.port = port
        heuristic = str(self.heuristic)
        env = os.environ.copy()
        # env["RUST_LOG"] = "debug,hyper=warn"
        self.process = subprocess.Popen(
            args=["../bamboozle_snake/target/release/bamboozle_snake", "--timeout", f"{timeout}", "--port", f"{port}",
                  "--duel-heuristic", heuristic, "--threads-per-game", "1", "--name", f"{self.name}"],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, env=env)

    def stop_server(self):
        self.process.kill()
        self.process = None
        self.port = None
