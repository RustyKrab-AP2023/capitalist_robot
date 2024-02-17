# Capitalist Robot

The Capitalist Robot is a program designed to explore the surrounding environment in search of resources such as rocks, garbage, and fish. Once found, the robot converts these resources into coins and deposits them at a local bank to accumulate wealth.

## Modes

The robot has the following modes:

- **Searching Content**: The robot explores the environment in search of rocks, garbage, and fish and put them in the backpack.

- **Scan Content**: Every x ticks in which the robot is searching content it uses a tool that scans the area around the robot in a cross pattern.
  
- **Follow Street**: If the robot encounters a street during its exploration, it will follow it until it leads to a city.

- **Scan Bank**: When the robot finds a city it scans the area around it and checks if there is a bank, if there's one the robot saves the coordinates.

- **Seaching Bank**: When the backpack is full the robots go to the nearest available bank it saved. If there are no available banks saved the robot uses a tool that allows the robot to explore a certain area in order to find a bank.


The Capitalist Robot was created by [Matteo Parma] (https://github.com/teoparma).
