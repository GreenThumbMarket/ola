# Recursion Feature in Ola CLI

## Overview
The recursion feature allows Ola to execute multiple waves of a command in sequence, with each wave being aware of its position in the recursion stack. This enables complex multi-step thinking processes that build upon previous results.

## Usage
To use the recursion feature, add the `-r` flag followed by a number from 1 to 10 to your command:

```bash
ola prompt -g "Your prompt here" -r 3
```

This will execute the prompt command 3 times in sequence, with each execution being aware of its recursion level.

## How it works
1. When you specify `-r N`, Ola will execute up to N recursive waves of the command
2. Each wave runs as a separate process with the same arguments as the original command
3. Waves are tracked using the `OLA_RECURSION_WAVE` environment variable
4. Each wave displays a color-coded indicator showing its level in the recursion stack

## Visual Indicators
Each recursion wave is color-coded for easy identification:
- Wave 1: Red
- Wave 2: Yellow
- Wave 3: Green
- Wave 4: Cyan
- Wave 5: Blue
- And so on...

## Logging
When logging is enabled, each recursion wave will include its wave number in the log entry, making it easy to track the flow of recursive executions.

## Use Cases
- Multi-step reasoning processes
- Building on previous outputs
- Implementing iterative refinement of results
- Creating feedback loops where model output guides subsequent prompts

## Limitations
- Maximum recursion depth is capped at 10 levels to prevent infinite loops
- Each recursion wave executes the same command with the same arguments
- Data is not automatically passed between recursion waves (future enhancement)

## Future Enhancements
- Ability to pass data between recursion waves
- Custom recursion templates
- Branch and conditional recursion based on output