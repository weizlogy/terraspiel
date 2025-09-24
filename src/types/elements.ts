export type ElementName = 
  | 'EMPTY'
  | 'SOIL'
  | 'WATER'
  | 'FIRE'
  | 'WET_SOIL'
  | 'STEAM';

export interface Element {
  name: ElementName;
  color: string;
  density: number; // For physics simulation
  isStatic?: boolean; // Whether the element is static or moves
  lifespan?: number; // For elements that change over time
  alpha?: number; // Optional alpha value for rendering
}

export const ELEMENTS: Record<ElementName, Element> = {
  EMPTY: { name: 'EMPTY', color: '#000000', density: 0, isStatic: true },
  SOIL: { name: 'SOIL', color: '#8B4513', density: 2, isStatic: false },
  WATER: { name: 'WATER', color: '#4169E1', density: 1, isStatic: false, alpha: 0.7 },
  FIRE: { name: 'FIRE', color: '#FF4500', density: 0.5, isStatic: false, lifespan: 100 },
  WET_SOIL: { name: 'WET_SOIL', color: '#654321', density: 2, isStatic: false },
  STEAM: { name: 'STEAM', color: '#C0C0C0', density: 0.1, isStatic: false },
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