export type ElementName =
  | 'EMPTY'
  | 'SOIL'
  | 'WATER'
  | 'MUD';

export interface Element {
  name: ElementName;
  color: string;
  density: number; // For physics simulation
  isStatic?: boolean; // Whether the element is static or moves
  lifespan?: number; // For elements that change over time
  alpha?: number; // Optional alpha value for rendering
}

export interface Cell {
  type: ElementName;
  counter?: number; // Optional counter for transformations
}

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
  consumes?: ElementName; // Optional: The element to consume from a neighbor upon transformation
}

export const ELEMENTS = {
  EMPTY: { name: 'EMPTY', color: '#000000', density: 0, isStatic: true },
  SOIL: { name: 'SOIL', color: '#8B4513', density: 4, isStatic: false },
  WATER: { name: 'WATER', color: '#1E90FF', density: 3, isStatic: false },
  MUD: { name: 'MUD', color: '#5D4037', density: 3.5, isStatic: false },
};

// Represents a particle with floating point coordinates and velocity
export interface Particle {
  id: number;
  px: number; // x-position
  py: number; // y-position
  vx: number; // x-velocity
  vy: number; // y-velocity
  type: ElementName;
  life: number; // Lifespan of the particle
}

export type MoveDirection = 'NONE' | 'DOWN' | 'DOWN_LEFT' | 'DOWN_RIGHT' | 'LEFT' | 'RIGHT';