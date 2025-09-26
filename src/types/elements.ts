export type ElementName =
  | 'EMPTY'
  | 'SOIL'
  | 'WATER'
  | 'MUD'
  | 'FERTILE_SOIL'
  | 'PEAT'
  | 'CLOUD'
  | 'CLAY';

export type ParticleType = ElementName | 'ETHER';

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

export type ConditionType = 'surrounding' | 'environment';

export interface BaseCondition {
  type: ConditionType;
  element: ElementName;
}

export interface SurroundingCondition extends BaseCondition {
  type: 'surrounding';
  min?: number;
  max?: number;
}

export interface EnvironmentCondition extends BaseCondition {
  type: 'environment';
  presence: 'exists' | 'not_exists';
  radius: number;
}

export type RuleCondition = SurroundingCondition | EnvironmentCondition;

export interface TransformationRule {
  from: ElementName;
  to: ElementName;
  probability: number;
  threshold: number;
  conditions: RuleCondition[];
  consumes?: ElementName; // Optional: The element to consume from a neighbor upon transformation
}

export const ELEMENTS = {
  EMPTY: { name: 'EMPTY', color: '#000000', density: 0, isStatic: true },
  SOIL: { name: 'SOIL', color: '#8B4513', density: 4, isStatic: false },
  WATER: { name: 'WATER', color: '#1E90FF', density: 3, isStatic: false },
  MUD: { name: 'MUD', color: '#4E342E', density: 3.5, isStatic: false },
  FERTILE_SOIL: { name: 'FERTILE_SOIL', color: '#5C4033', density: 4, isStatic: false },
  PEAT: { name: 'PEAT', color: '#3E2723', density: 4, isStatic: false },
  CLOUD: { name: 'CLOUD', color: '#F0F8FF', density: 2, isStatic: false, alpha: 0.9 },
  CLAY: { name: 'CLAY', color: '#BCAAA4', density: 4.2, isStatic: false },
};

// Represents a particle with floating point coordinates and velocity
export interface Particle {
  id: number;
  px: number; // x-position
  py: number; // y-position
  vx: number; // x-velocity
  vy: number; // y-velocity
  type: ParticleType;
  life: number; // Lifespan of the particle
}

export type MoveDirection = 'NONE' | 'DOWN' | 'DOWN_LEFT' | 'DOWN_RIGHT' | 'LEFT' | 'RIGHT';