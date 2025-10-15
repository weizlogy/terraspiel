import Phaser from 'phaser';
import useGameStore from '../stores/gameStore';
import { type ElementName, type Particle, type MoveDirection, type Cell, type Element } from '../types/elements';
import { simulateWorld, calculateStats } from './physics';
import { blendColors } from '../utils/colors';

const PARTICLE_ELEMENTS: ElementName[] = [];

export class GameScene extends Phaser.Scene {
  private grids: [Cell[][], Cell[][]] = [[], []];
  private lastMoveGrids: [MoveDirection[][], MoveDirection[][]] = [[], []];
  private colorGrids: [string[][], string[][]] = [[], []];
  private activeBufferIndex: 0 | 1 = 0;

  private particles: Particle[] = []; // Add state for particles
  private elements: Record<ElementName, Element> = {} as Record<ElementName, Element>;
  private width: number = 160; // Fixed grid width
  private height: number = 90; // Fixed grid height
  private cellSize: number = 4; // Fixed cell size
  private gridGraphics!: Phaser.GameObjects.Graphics;
  private isDrawing: boolean = false;
  private lastSimulationTime: number = 0;
  private simulationInterval: number = 30; // ms - even faster physics updates for smoother movement
  private lastDrawTime: number = 0;
  private drawInterval: number = 30; // ms - more responsive drawing when holding mouse
  private frameCount: number = 0;

  constructor() {
    super('GameScene');
  }

  init() {
    // Get initial state from store
    const state = useGameStore.getState();
    // Initialize both buffers
    this.grids[0] = state.grid.map(row => row.map(cell => ({ ...cell })));
    this.grids[1] = state.grid.map(row => row.map(cell => ({ ...cell })));
    this.lastMoveGrids[0] = state.lastMoveGrid.map(row => [...row]);
    this.lastMoveGrids[1] = state.lastMoveGrid.map(row => [...row]);
    this.colorGrids[0] = state.colorGrid.map(row => [...row]);
    this.colorGrids[1] = state.colorGrid.map(row => [...row]);

    this.particles = state.particles;
    this.elements = state.elements;
    this.width = state.width;
    this.height = state.height;
  }

  create() {
    // Create graphics object for drawing the grid and particles
    this.gridGraphics = this.add.graphics();
    
    // Set up input handling
    this.input.on('pointerdown', this.handlePointerDown, this);
    this.input.on('pointermove', this.handlePointerMove, this);
    this.input.on('pointerup', this.handlePointerUp, this);
    
    // Subscribe to store updates for grid and particles
    useGameStore.subscribe((state) => {
      // Only sync grid data if the update is NOT from the simulation loop
      if (state.updateSource !== 'simulation') {
        this.grids[0] = state.grid.map(row => row.map(cell => ({ ...cell })));
        this.grids[1] = state.grid.map(row => row.map(cell => ({ ...cell })));
        this.lastMoveGrids[0] = state.lastMoveGrid.map(row => [...row]);
        this.lastMoveGrids[1] = state.lastMoveGrid.map(row => [...row]);
        this.colorGrids[0] = state.colorGrid.map(row => [...row]);
        this.colorGrids[1] = state.colorGrid.map(row => [...row]);
      }

      this.particles = state.particles;
      this.elements = state.elements;
      this.width = state.width;
      this.height = state.height;
      this.renderAll(); // Re-render whenever state changes
    });
    
    // Initial render
    this.renderAll();
  }
  
  update(time: number) {
    // Update FPS counter
    const fps = Math.round(this.game.loop.actualFps);
    useGameStore.getState().setFps(fps);
    
    // Run physics simulation at intervals
    if (time - this.lastSimulationTime > this.simulationInterval) {
      this.lastSimulationTime = time;
      this.runSimulation();
    }
    
    // Handle continuous drawing when mouse is held down
    if (this.isDrawing) {
      const pointer = this.input.activePointer;
      if (pointer.isDown && time - this.lastDrawTime > this.drawInterval) {
        this.lastDrawTime = time;
        this.updateAtPointer(pointer.x, pointer.y);
      }
    }
  }

  private runSimulation() {
    const readBufferIndex = this.activeBufferIndex;
    const writeBufferIndex = 1 - this.activeBufferIndex;

    // Simulate grid and particles together, writing to the back buffer
    const { newParticles } = simulateWorld(
      this.grids[readBufferIndex],
      this.lastMoveGrids[readBufferIndex],
      this.colorGrids[readBufferIndex],
      this.grids[writeBufferIndex],
      this.lastMoveGrids[writeBufferIndex],
      this.colorGrids[writeBufferIndex],
      this.particles,
      this.frameCount
    );

    this.particles = newParticles;

    // Swap buffers for the next frame
    this.activeBufferIndex = writeBufferIndex as 0 | 1;
    this.frameCount++;

    // The write buffers are now the new read buffers. We can update the store.
    const state = useGameStore.getState();
    state.setSimulationResult({
      newGrid: this.grids[this.activeBufferIndex],
      newLastMoveGrid: this.lastMoveGrids[this.activeBufferIndex],
      newColorGrid: this.colorGrids[this.activeBufferIndex],
      newParticles: newParticles,
    });
    
    // Update stats from the new grid and particles
    const stats = calculateStats(this.grids[this.activeBufferIndex], newParticles);
    state.updateStats(stats);
  }

  private handlePointerDown(pointer: Phaser.Input.Pointer) {
    if (pointer.rightButtonDown()) return; // Ignore right-clicks
    this.isDrawing = true;
    this.updateAtPointer(pointer.x, pointer.y);
  }

  private handlePointerMove(pointer: Phaser.Input.Pointer) {
    if (this.isDrawing) {
      this.updateAtPointer(pointer.x, pointer.y);
    }
  }

  private handlePointerUp() {
    this.isDrawing = false;
  }

  private updateAtPointer(x: number, y: number) {
    const gridX = Math.floor(x / this.cellSize);
    const gridY = Math.floor(y / this.cellSize);
    
    // Check bounds
    if (gridX < 0 || gridX >= this.width || gridY < 0 || gridY >= this.height) {
      return;
    }

    const readBuffer = this.grids[this.activeBufferIndex];

    // Only allow drawing on empty cells
    if (readBuffer[gridY][gridX].type !== 'EMPTY') {
      return;
    }
    
    const { selectedElement, elements, colorVariations } = useGameStore.getState();
    const elementType = selectedElement as ElementName;

    // If selected element is a particle type, add a particle. Otherwise, update the grid.p
    if (PARTICLE_ELEMENTS.includes(elementType)) {
      // Add particle with some initial random velocity
      const vx = (Math.random() - 0.5) * 0.5;
      const vy = (Math.random() - 0.5) * 0.5;
      useGameStore.getState().addParticle(gridX + 0.5, gridY + 0.5, elementType, vx, vy);
    } else {
      const elementInfo = elements[elementType];
      if (!elementInfo) {
        console.error(`Selected element '${elementType}' does not exist in elements dictionary`);
        return;
      }
      
      const newCell = { type: elementType };
      let newColor = elementInfo.color;

      if (elementInfo.hasColorVariation) {
        const variations = colorVariations.get(elementType);
        if (variations && variations.length > 0) {
          newColor = variations[Math.floor(Math.random() * variations.length)];
        }
      }

      // Directly modify both buffers to ensure consistency
      // This prevents the simulation from overwriting the new particle immediately
      this.grids[0][gridY][gridX] = newCell;
      this.grids[1][gridY][gridX] = newCell;
      this.colorGrids[0][gridY][gridX] = newColor;
      this.colorGrids[1][gridY][gridX] = newColor;

      // We don't need to update stats here anymore because the simulation loop will do it
    }
  }

  private renderAll() {
    const readGrid = this.grids[this.activeBufferIndex];
    const readColorGrid = this.colorGrids[this.activeBufferIndex];

    // Add a guard clause to check if the grid is initialized
    if (!readGrid || readGrid.length === 0 || !readGrid[0] || readGrid[0].length === 0) {
      return;
    }
    if (!this.elements || Object.keys(this.elements).length === 0) return; // Existing guard
    this.gridGraphics.clear();
    
    // 1. Render the grid cells
    for (let y = 0; y < this.height; y++) {
      for (let x = 0; x < this.width; x++) {
        const elementName = readGrid[y][x].type;
        if (elementName !== 'EMPTY') {
          const displayColor = readColorGrid[y][x];

          this.gridGraphics.fillStyle(parseInt(displayColor.replace('#', '0x')), 1);
          this.gridGraphics.fillRect(
            x * this.cellSize,
            y * this.cellSize,
            this.cellSize,
            this.cellSize
          );
        }
      }
    }
    
    // 2. Render the particles
    for (const particle of this.particles) {
      const particleType = particle.type;
      if (particleType === 'EMPTY') {
        continue;
      }

      if (particleType === 'ETHER') {
        const baseAlpha = Math.max(0, particle.life / 150) * 0.5; // More transparent
        const radius = this.cellSize * 0.5; // Larger radius
        const color = 0xFFFFFF; // White

        // Draw a single circle for performance
        this.gridGraphics.fillStyle(color, baseAlpha);
        this.gridGraphics.fillCircle(particle.px * this.cellSize, particle.py * this.cellSize, radius);

      } else if (particleType === 'THUNDER') {
        const baseAlpha = Math.max(0, particle.life / 20); // Fade out as it dies
        const radius = this.cellSize * 0.5;
        const color = 0xFFFF00; // Yellow

        this.gridGraphics.fillStyle(color, baseAlpha);
        this.gridGraphics.fillCircle(
          particle.px * this.cellSize,
          particle.py * this.cellSize,
          radius
        );
      } else if (particleType === 'FIRE') {
        const element = this.elements.FIRE;
        if (element) {
          // 1. Color variation (from orange to yellow)
          const baseColor = Phaser.Display.Color.ValueToColor(element.color);
          const yellowColor = Phaser.Display.Color.ValueToColor('#FFFF00');
          const blended = Phaser.Display.Color.Interpolate.ColorWithColor(baseColor, yellowColor, 100, Math.floor(Math.random() * 70));
          const color = Phaser.Display.Color.GetColor(blended.r, blended.g, blended.b);

          // 2. Transparency based on life (fades out)
          const alpha = Math.min(0.9, Math.max(0.1, particle.life / 90));

          // 3. Flicker/Wobble effect
          const wobbleX = (Math.random() - 0.5) * 0.5; // small random offset
          const wobbleY = (Math.random() - 0.5) * 0.5; // small random offset
          
          const radius = this.cellSize * (0.5 + Math.random() * 0.2); // Varying radius

          this.gridGraphics.fillStyle(color, alpha);
          this.gridGraphics.fillCircle(
            (particle.px + wobbleX) * this.cellSize,
            (particle.py + wobbleY) * this.cellSize,
            radius
          );
        }
      } else {
        const element = this.elements[particleType as ElementName];
        if (element) {
          const color = parseInt(element.color.replace('#', '0x'));
          const radius = this.cellSize * 0.5;
          this.gridGraphics.fillStyle(color, 1.0);
          this.gridGraphics.fillCircle(
            particle.px * this.cellSize,
            particle.py * this.cellSize,
            radius
          );
        }
      }
    }
  }
}