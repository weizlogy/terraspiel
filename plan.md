# Terraspiel Development Plan

## Feature: Probabilistic Transformation of Pixels

This feature will introduce a mechanism where pixels can change into other substances based on a set of conditions including surrounding pixels, time, and probability.

### 1. Evolve Pixel Data Structure

- **Problem:** The current grid (`ElementName[][]`) is an array of strings, which is insufficient for storing a "change counter".
- **Solution:**
    - Introduce a new `Cell` interface in `src/types/elements.ts`.
      ```typescript
      export interface Cell {
        type: ElementName;
        counter?: number; // Optional counter for transformations
      }
      ```
    - Update the main grid in `gameStore.ts` and `physics.ts` to be `Cell[][]`. This is a fundamental change affecting many parts of the code.

### 2. Create a Generic Transformation Behavior

- **Solution:**
    - Create a new file: `src/game/transformation.ts` (at the same level as `physics.ts`).
    - This file will contain a function, `handleTransformations`, that runs *after* the primary movement behavior for a pixel.
    - This function will:
        a. Accept the `BehaviorContext` (which will be updated to use `Cell[][]`).
        b. Accept a set of `TransformationRule`s.
        c. For each rule applicable to the current cell's `type`:
            i. Check conditions (surrounding cell types and counts).
            ii. If conditions are met, increment the `counter` on the `Cell` object with a given probability (`Math.random() < probability`).
            iii. If the `counter` exceeds the rule's `threshold`, transform the cell's `type` to the `resultElement` and reset the counter.

### 3. Define Transformation Rules

- **Solution:**
    - Create a new file, `src/game/rules.ts`, to define the transformation rules in a structured way. This makes it easy to add or change transformations without digging into the logic.
    
      ```typescript
      // In src/game/rules.ts
      export interface TransformationRule {
        from: ElementName;
        to: ElementName;
        probability: number;
        threshold: number;
        conditions: {
          surrounding: {
            type: ElementName;
            min?: number;
            max?: number;
          }[];
        };
      }

      export const transformationRules: TransformationRule[] = [
        {
          from: 'HAY',
          to: 'TINDER',
          probability: 0.1, // 10% chance each step to increment counter
          threshold: 100,   // Transforms when counter reaches 100
          conditions: {
            surrounding: [
              
            ]
          }
        }
      ];
      ```

### 4. Update Physics Simulation

- **Solution:**
    - In `src/game/physics.ts`, modify `simulatePhysics` to:
        a. Work with the new `Cell[][]` grid.
        b. After a pixel's primary behavior (like `handleSoil`) is executed, call `handleTransformations` for that same pixel. This ensures movement and transformation can both be calculated in the same simulation step.

### 5. Refactor Existing Code

- **Solution:**
    - Update `App.tsx`, `gameStore.ts`, all behavior files (`soilBehavior.ts`, etc.), and the rendering logic to handle the new `Cell[][]` grid structure. This will mostly involve changing `grid[y][x]` to `grid[y][x].type` where the element's name is needed.