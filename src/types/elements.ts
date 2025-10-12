export type ElementName =
  | 'EMPTY'
  | 'SOIL'
  | 'WATER'
  | 'MUD'
  | 'FERTILE_SOIL'
  | 'PEAT'
  | 'CLOUD'
  | 'CLAY'
  | 'FIRE'
  | 'SAND'
  | 'STONE'
  | 'BASALT'
  | 'OBSIDIAN'
  | 'SEED'
  | 'PLANT'
  | 'OIL'
  | 'CRYSTAL'
  | 'ELECTRUM'
  | 'RUBY'
  | 'SAPPHIRE'
  | 'AMETHYST'
  | 'GARNET'
  | 'EMERALD'
  | 'ETHER'
  | 'THUNDER';

export type ParticleType = ElementName;

export interface Element {
  name: ElementName;
  color: string;
  density: number; // For physics simulation
  state?: 'solid' | 'liquid';
  isStatic?: boolean; // Whether the element is static or moves
  lifespan?: number; // For elements that change over time
  alpha?: number; // Optional alpha value for rendering
  fluidity?: {
    resistance: number; // Chance to resist diagonal movement (0-1)
    spread: number;     // Chance to spread horizontally (0-1)
  };
  [key: string]: any;
}

export interface Cell {
  type: ElementName;
  rainCounter?: number; // for CLOUD
  chargeCounter?: number; // for CLOUD
  decayCounter?: number; // for CLOUD & PLANT
  rainThreshold?: number; // for CLOUD
  chargeThreshold?: number; // for CLOUD
  counter?: number; // Optional counter for transformations & growth
  burningProgress?: number; // For combustion progress tracking
  plantMode?: 'stem' | 'ground_cover' | 'leaf' | 'flower' | 'withered'; // for PLANT
  oilCounter?: number; // for withered PLANT to turn into OIL
  life?: number; // for FIRE
  etherStorage?: number; // for CRYSTAL
}

export type ConditionType = 'surrounding' | 'environment' | 'surroundingAttribute';

export interface BaseCondition {
  type: ConditionType;
  element?: ElementName;
}

export interface SurroundingCondition extends BaseCondition {
  type: 'surrounding';
  element: ElementName;
  min?: number;
  max?: number;
}

export interface EnvironmentCondition extends BaseCondition {
  type: 'environment';
  element: ElementName;
  presence: 'exists' | 'not_exists';
  radius: number;
}

export interface SurroundingAttributeCondition extends BaseCondition {
  type: 'surroundingAttribute';
  attribute: string;
  value: any;
  min?: number;
  max?: number;
}

export type RuleCondition = SurroundingCondition | EnvironmentCondition | SurroundingAttributeCondition;

export interface TransformationRule {
  from: ElementName;
  to: ElementName;
  probability: number;
  threshold: number;
  conditions: RuleCondition[];
  consumes?: ElementName; // Optional: The element to consume from a neighbor upon transformation
  spawnParticle?: ElementName; // Optional: The particle to spawn upon transformation
}



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
