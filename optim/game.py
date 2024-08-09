import subprocess
import threading
from typing import List

from snake import Snake


class Game(threading.Thread):
    def __init__(self, snakes: List[Snake], browser: bool = False, sequential: bool = True):
        super(Game, self).__init__()
        self.process = None
        self.browser = browser
        self.sequential = sequential
        self.snakes = snakes
        self.winner = None

    def run(self):
        args = ["bamboozle_snake", "play", "-m","royale", "-n", f"{self.snakes[0].name}", "-n", f"{self.snakes[1].name}", "-u",
                f"http://127.0.0.1:{self.snakes[0].port}/", "-u", f"http://127.0.0.1:{self.snakes[1].port}/"]

        if self.browser:
            args.append("--browser")

        if self.sequential:
            args.append("-s")

        self.process = subprocess.Popen(args=args, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        lines = []
        while True:
            line = self.process.stderr.readline().decode("utf-8")
            if "Error" in line:
                print(line)
                
            if line == "":
                print("WARN")
                self.winner = "DRAW"
                return
            split = line.split()
            if len(split) > 2 and split[2] == "Game":
                break
        self.process.kill()
        self.process = None

        winner = line.split()[-4]
        if winner == "It":
            self.winner = "DRAW"
        else:
            self.winner = winner
