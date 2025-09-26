import { type TransformationRule } from "../types/elements";

export const transformationRules: TransformationRule[] = [
  {
    from: 'SOIL',
    to: 'MUD',
    probability: 0.1, // 10% chance per step
    threshold: 10,    // after 10 counter increments
    conditions: {
      surrounding: [
        { type: 'WATER', min: 1 }, // if at least 1 water is nearby
      ]
    },
    consumes: 'WATER' // Consume one water neighbor on transformation
  }
];
