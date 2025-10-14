import Phaser from 'phaser';
import useGameStore from '../stores/gameStore';
import { type ElementName, type Particle, type MoveDirection, type Cell, type Element } from '../types/elements';
import { simulateWorld, calculateStats } from './physics';
import { varyColor } from '../utils/colors';

const PARTICLE_ELEMENTS: ElementName[] = [];

export class GameScene extends Phaser.Scene {
  // Buffers for simulation
  private grids: [Cell[][], Cell[][]] = [[], []];
  private lastMoveGrids: [MoveDirection[][], MoveDirection[][]] = [[], []];
  private colorGrids: [string[][], string[][]] = [[], []];
  private activeBufferIndex: 0 | 1 = 0;

  // State
  private particles: Particle[] = [];
  private elements: Record<ElementName, Element> = {} as Record<ElementName, Element>;
  private width: number = 320;
  private height: number = 180;
  private cellSize: number = 4;
  private frameCount: number = 0;

  // Drawing & Performance
  private gridTexture!: Phaser.GameObjects.RenderTexture;
  private particleGraphics!: Phaser.GameObjects.Graphics;
  private cellGraphics!: Phaser.GameObjects.Graphics; // Added for cell drawing
  private dirtyCells: Set<string> = new Set();
  private needsFullRedraw: boolean = true;

  // Input & Timing
  private isDrawing: boolean = false;
  private lastSimulationTime: number = 0;
  private simulationInterval: number = 30; 
  private lastDrawTime: number = 0;
  private drawInterval: number = 30;

  constructor() {
    super('GameScene');
  }

  init() {
    const state = useGameStore.getState();
    this.grids[0] = (state.grid || []).map(row => row.map(cell => ({ ...cell })));
    this.grids[1] = (state.grid || []).map(row => row.map(cell => ({ ...cell })));
    this.lastMoveGrids[0] = (state.lastMoveGrid || []).map(row => [...row]);
    this.lastMoveGrids[1] = (state.lastMoveGrid || []).map(row => [...row]);
    this.colorGrids[0] = (state.colorGrid || []).map(row => [...row]);
    this.colorGrids[1] = (state.colorGrid || []).map(row => [...row]);
    this.particles = state.particles || [];
    this.elements = state.elements || {};
    this.width = state.width;
    this.height = state.height;
  }

  create() {
    // Use a RenderTexture for the main grid, allowing for efficient partial updates.
    this.gridTexture = this.add.renderTexture(0, 0, this.width * this.cellSize, this.height * this.cellSize);
    // Use a separate Graphics object for particles, which are cleared and redrawn each frame.
    this.particleGraphics = this.add.graphics();
    this.cellGraphics = this.add.graphics(); // Initialize cell graphics

    this.input.on('pointerdown', this.handlePointerDown, this);
    this.input.on('pointermove', this.handlePointerMove, this);
    this.input.on('pointerup', this.handlePointerUp, this);

    useGameStore.subscribe((state) => {
      if (state.updateSource !== 'simulation') {
        this.grids[0] = (state.grid || []).map(row => row.map(cell => ({ ...cell })));
        this.grids[1] = (state.grid || []).map(row => row.map(cell => ({ ...cell })));
        this.lastMoveGrids[0] = (state.lastMoveGrid || []).map(row => [...row]);
        this.lastMoveGrids[1] = (state.lastMoveGrid || []).map(row => [...row]);
        this.colorGrids[0] = (state.colorGrid || []).map(row => [...row]);
        this.colorGrids[1] = (state.colorGrid || []).map(row => [...row]);
        this.needsFullRedraw = true; // Force a full redraw on external changes
      }
      this.particles = state.particles || [];
      this.elements = state.elements || {};
      this.width = state.width;
      this.height = state.height;
    });

    this.initialDraw();
  }

  update(time: number) {
    const fps = Math.round(this.game.loop.actualFps);
    useGameStore.getState().setFps(fps);

    if (time - this.lastSimulationTime > this.simulationInterval) {
      this.lastSimulationTime = time;
      this.runSimulation();
      this.draw(); // Draw after simulation
    }

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

    const readGrid = this.grids[readBufferIndex];
    const writeGrid = this.grids[writeBufferIndex];

    // Guard against uninitialized grids
    if (!readGrid || readGrid.length === 0 || !writeGrid || writeGrid.length === 0) {
        return;
    }

    const { newParticles } = simulateWorld(
      readGrid,
      this.lastMoveGrids[readBufferIndex],
      this.colorGrids[readBufferIndex],
      writeGrid,
      this.lastMoveGrids[writeBufferIndex],
      this.colorGrids[writeBufferIndex],
      this.particles,
      this.frameCount
    );

    // Detect dirty cells by comparing read and write buffers
    for (let y = 0; y < this.height; y++) {
      for (let x = 0; x < this.width; x++) {
        if (readGrid[y][x].type !== writeGrid[y][x].type) {
          this.dirtyCells.add(`${x},${y}`);
        }
      }
    }

    this.particles = newParticles;
    this.activeBufferIndex = writeBufferIndex as 0 | 1;
    this.frameCount++;

    const state = useGameStore.getState();
    state.setSimulationResult({
      newGrid: this.grids[this.activeBufferIndex],
      newLastMoveGrid: this.lastMoveGrids[this.activeBufferIndex],
      newColorGrid: this.colorGrids[this.activeBufferIndex],
      newParticles: newParticles,
    });

    const stats = calculateStats(this.grids[this.activeBufferIndex], newParticles);
    state.updateStats(stats);
  }

  private handlePointerDown(pointer: Phaser.Input.Pointer) {
    if (pointer.rightButtonDown()) return;
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

    if (gridX < 0 || gridX >= this.width || gridY < 0 || gridY >= this.height) return;

    if (this.grids[this.activeBufferIndex][gridY][gridX].type !== 'EMPTY') return;
    
    const state = useGameStore.getState();
    const selectedElement = state.selectedElement as ElementName;

    if (PARTICLE_ELEMENTS.includes(selectedElement)) {
      const vx = (Math.random() - 0.5) * 0.5;
      const vy = (Math.random() - 0.5) * 0.5;
      state.addParticle(gridX + 0.5, gridY + 0.5, selectedElement, vx, vy);
    } else {
      if (!this.elements[selectedElement]) return;
      
      const newCell = { type: selectedElement };
      const baseColor = this.elements[selectedElement].color;
      const newColor = selectedElement !== 'EMPTY' ? varyColor(baseColor) : baseColor;

      this.grids[0][gridY][gridX] = newCell;
      this.grids[1][gridY][gridX] = newCell;
      this.colorGrids[0][gridY][gridX] = newColor;
      this.colorGrids[1][gridY][gridX] = newColor;

      this.dirtyCells.add(`${gridX},${gridY}`);
    }
  }

  private draw() {
    if (this.needsFullRedraw) {
      this.initialDraw();
      this.needsFullRedraw = false;
    } else {
      this.drawDirtyCells();
    }
    this.drawParticles();
  }

  private initialDraw() {
    this.gridTexture.clear();
    const readGrid = this.grids[this.activeBufferIndex];
    const readColorGrid = this.colorGrids[this.activeBufferIndex];

    if (!readGrid || readGrid.length === 0 || !this.elements || Object.keys(this.elements).length === 0) return;

    this.cellGraphics.clear();
    for (let y = 0; y < this.height; y++) {
      for (let x = 0; x < this.width; x++) {
        const elementName = readGrid[y][x].type;
        if (elementName !== 'EMPTY') {
          const displayColor = readColorGrid[y][x];
          const color = parseInt(displayColor.replace('#', '0x'));
          this.cellGraphics.fillStyle(color, 1);
          this.cellGraphics.fillRect(x * this.cellSize, y * this.cellSize, this.cellSize, this.cellSize);
        }
      }
    }
    this.gridTexture.draw(this.cellGraphics);
  }

  private drawDirtyCells() {
    if (this.dirtyCells.size === 0) return;

    const readGrid = this.grids[this.activeBufferIndex];
    const readColorGrid = this.colorGrids[this.activeBufferIndex];

    this.cellGraphics.clear();

    // Batch erase
    this.dirtyCells.forEach(key => {
      const [x, y] = key.split(',').map(Number);
      this.cellGraphics.fillRect(x * this.cellSize, y * this.cellSize, this.cellSize, this.cellSize);
    });
    this.gridTexture.erase(this.cellGraphics);
    this.cellGraphics.clear();

    // Batch draw
    this.dirtyCells.forEach(key => {
      const [x, y] = key.split(',').map(Number);
      const elementName = readGrid[y][x].type;
      if (elementName !== 'EMPTY') {
        const displayColor = readColorGrid[y][x];
        const color = parseInt(displayColor.replace('#', '0x'));
        this.cellGraphics.fillStyle(color, 1);
        this.cellGraphics.fillRect(x * this.cellSize, y * this.cellSize, this.cellSize, this.cellSize);
      }
    });
    this.gridTexture.draw(this.cellGraphics);

    this.dirtyCells.clear();
  }

  private drawParticles() {
    this.particleGraphics.clear();
    if (!this.elements || Object.keys(this.elements).length === 0) return;

    for (const particle of this.particles) {
      const particleType = particle.type;
      if (particleType === 'EMPTY') continue;

      const element = this.elements[particleType as ElementName];
      if (!element) continue;

      const alpha = (element.name === 'ETHER' || element.name === 'THUNDER' || element.name === 'FIRE') 
        ? Math.max(0, particle.life / (element.name === 'ETHER' ? 150 : (element.name === 'THUNDER' ? 20 : 90))) 
        : 1.0;

      const color = parseInt(element.color.replace('#', '0x'));
      const radius = this.cellSize * 0.5;

      this.particleGraphics.fillStyle(color, alpha);
      this.particleGraphics.fillCircle(
        particle.px * this.cellSize,
        particle.py * this.cellSize,
        radius
      );
    }
  }
}
