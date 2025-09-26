# Terraspiel - Alchemy Pixel Playground

A pixel-based physics simulation game with alchemy elements, similar to sandspiel.club.

## Concept

This is a falling sand game combined with alchemy mechanics where players can combine elements to create new ones.

## Technology Stack

- Frontend UI: React + TypeScript + Zustand + Tailwind CSS
- Game Engine: Phaser 3 (WebGL mode)
- Physics/Synthesis Logic: Custom grid physics + ReactionRule manager (implemented in TypeScript)
- Rendering Optimization: Phaser DynamicTexture + Custom shaders (as needed)
- Data Persistence: Zustand + localStorage (extension: Firebase)
- Sound: Howler.js
- Build/Bundling: Vite (fast HMR + production optimization)
- Hosting: Vercel / Netlify (can be deployed as a static site)

## Setup

1. Clone the repository
2. Run `npm install` to install dependencies
3. Run `npm run dev` to start the development server

## Features Implemented

- Basic grid-based physics engine
- Element placement with mouse
- Toolbar for element selection
- Play/Pause functionality
- Statistics panel
- Soil physics (gravity)
- Water physics (flowing)

## Planned Features


- Plant growth lifecycle logic
- Alchemy reaction rules
- Additional elements (wet soil, sand, etc.)
- Complete debug information display
- Save/Load functionality