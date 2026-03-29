# Conway's Game of Life Simulator — SPEC.md

## 1. Concept & Vision

A mesmerizing, interactive simulation of Conway's Game of Life that transforms cellular automata into an art form. The interface combines a sleek dark aesthetic with vibrant neon cell animations, creating an immersive experience where users can paint life patterns directly onto a glowing grid. The feeling should be "digital petri dish meets cyberpunk laboratory."

## 2. Design Language

### Aesthetic Direction
Inspired by **synthwave laboratory** — dark backgrounds with glowing neon cells, CRT-style subtle scanlines, and ethereal color gradients. The grid should feel alive, with cells pulsing gently as they exist.

### Color Palette
- **Background**: `#0a0a0f` (deep space black)
- **Grid Lines**: `#1a1a2e` (subtle purple-tinted dark)
- **Dead Cell**: `#12121a` (near-black)
- **Alive Cell - Primary**: `#00ff88` (neon mint green)
- **Alive Cell Glow**: `#00ff88` with 40% opacity blur
- **Accent Hot**: `#ff006e` (hot pink for actions)
- **Accent Cool**: `#00d4ff` (cyan for info)
- **Text Primary**: `#e0e0e0`
- **Text Muted**: `#666680`

### Typography
- **Primary Font**: "JetBrains Mono" (monospace, technical feel)
- **Fallback**: "SF Mono", "Monaco", "Consolas", monospace
- **Headings**: 600 weight, letter-spacing: -0.02em
- **Body**: 400 weight

### Spatial System
- Base unit: 4px
- Spacing scale: 4, 8, 12, 16, 24, 32, 48, 64
- Grid cell size: 16px × 16px (configurable)
- Border radius: 4px (subtle), 8px (cards), 12px (buttons)

### Motion Philosophy
- Cell birth: scale 0→1, opacity 0→1, 150ms ease-out with glow pulse
- Cell death: opacity 1→0, scale 1→0.8, 100ms ease-in
- Generation transition: subtle wave effect across grid
- Button interactions: scale 0.98 on press, glow intensify on hover
- All transitions: 200ms cubic-bezier(0.4, 0, 0.2, 1)

### Visual Assets
- Icons: Lucide React (consistent stroke width)
- Grid: CSS-rendered with box-shadows for glow effects
- Background: subtle radial gradient from center
- Decorative: animated grid pulse on generation change

## 3. Layout & Structure

### Page Structure
```
┌─────────────────────────────────────────────────────────┐
│  HEADER: Logo + Title + Generation Counter              │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │                                                  │   │
│  │              INTERACTIVE GRID                   │   │
│  │         (Click to draw cells, 60x40)            │   │
│  │                                                  │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
├─────────────────────────────────────────────────────────┤
│  CONTROL PANEL                                          │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌──────────┐   │
│  │  Play    │ │  Pause   │ │  Step   │ │  Clear   │   │
│  └──────────┘ └──────────┘ └──────────┘ └──────────┘   │
│                                                         │
│  ┌──────────┐ ┌──────────┐ ┌──────────┐                 │
│  │  Random  │ │  Speed   │ │  Grid    │                 │
│  └──────────┘ └──────────┘ └──────────┘                 │
├─────────────────────────────────────────────────────────┤
│  FOOTER: Rules explanation + Population stats           │
└─────────────────────────────────────────────────────────┘
```

### Responsive Strategy
- Desktop (1200px+): Full grid, side controls possible
- Tablet (768px-1199px): Scaled grid, stacked controls
- Mobile (< 768px): Smaller grid (30x20), full-width controls

## 4. Features & Interactions

### Core Features

#### Grid Canvas
- **Size**: 60 columns × 40 rows (default)
- **Click**: Toggle single cell (alive ↔ dead)
- **Drag**: Paint cells as alive (when mouse down + moving)
- **Erase**: Right-click drag to erase cells
- **Hover**: Subtle highlight on cell under cursor

#### Game Controls
- **Play/Pause**: Toggle automatic generation advancement
- **Step**: Advance exactly one generation
- **Clear**: Reset grid to all dead cells
- **Randomize**: Fill grid with random cells (~30% alive)

#### Simulation Settings
- **Speed Slider**: 1-30 generations per second (default: 10)
- **Grid Size**: Preset buttons (Small 30×20, Medium 60×40, Large 80×50)

### Interaction Details

| Action | Behavior |
|--------|----------|
| Left click on cell | Toggle cell state |
| Left drag on grid | Paint cells alive |
| Right drag on grid | Erase cells |
| Hover on cell | Subtle border highlight |
| Click Play | Start auto-simulation, icon changes to Pause |
| Click Pause | Stop simulation, preserve state |
| Click Step | Advance 1 generation regardless of play state |
| Click Clear | Instant clear, reset generation to 0 |
| Click Random | Randomize with animation effect |
| Drag Speed slider | Real-time speed adjustment |

### Edge Cases
- Grid boundaries wrap toroidally (top↔bottom, left↔right)
- Speed change during play takes effect immediately
- Randomize during play continues simulation
- Clear during play resets but doesn't pause

## 5. Component Inventory

### GridCanvas
- **Default**: Dark cells with subtle grid lines
- **Cell Alive**: Neon green with glow shadow
- **Cell Dead**: Near-black, barely visible
- **Cell Hover**: White border (1px)
- **Cell Painting**: Trail effect as dragging

### ControlButton
- **Default**: Dark background (#1a1a2e), light text, subtle border
- **Hover**: Glow effect, slight scale up (1.02)
- **Active/Pressed**: Scale down (0.98), brighter glow
- **Disabled**: 50% opacity, no interactions
- **Playing State**: Green accent glow when simulation running

### SpeedSlider
- **Track**: Dark with subtle gradient
- **Thumb**: Cyan accent (#00d4ff) with glow
- **Hover**: Increased glow intensity
- **Labels**: Min "1 gen/s" to Max "30 gen/s"

### GenerationCounter
- **Display**: Large monospace numbers
- **Label**: "Generation" above
- **Animation**: Number tick-up effect on change

### PopulationDisplay
- **Format**: "Population: X / Y" (alive / total)
- **Style**: Compact, muted text, updates in real-time

### PresetSizeButtons
- **Default**: Ghost style, icon + label
- **Selected**: Filled with accent color
- **Hover**: Background highlight

## 6. Technical Approach

### Architecture
- **Backend**: Rust with Axum web framework
- **Frontend**: React 18 + TypeScript + Vite
- **Communication**: REST API (JSON)
- **State Management**: React hooks (useState, useEffect)

### API Design

#### Endpoints

```
GET  /api/state          → Current grid state
POST /api/init           → Initialize grid { width, height }
POST /api/toggle/{x}/{y} → Toggle cell { x, y }
POST /api/step           → Advance one generation
POST /api/randomize      → Randomize grid { density: 0.0-1.0 }
POST /api/clear          → Clear grid
```

#### Response Format
```json
{
  "width": 60,
  "height": 40,
  "cells": [[true, false, ...], ...],
  "generation": 0,
  "population": 150
}
```

### Data Model

#### Grid State
- `width: usize` — Number of columns
- `height: usize` — Number of rows
- `cells: Vec<Vec<bool>>` — 2D array of cell states (true = alive)
- `generation: u64` — Current generation number
- `population: usize` — Count of alive cells

### Conway's Rules Implementation
```rust
// Core rules applied simultaneously to all cells:
1. Any live cell with < 2 live neighbors → dies (underpopulation)
2. Any live cell with 2-3 live neighbors → survives
3. Any live cell with > 3 live neighbors → dies (overpopulation)
4. Any dead cell with exactly 3 neighbors → becomes alive (reproduction)
```

### Project Structure
```
lifegame/
├── Cargo.toml              # Rust dependencies
├── src/
│   ├── main.rs             # Entry point, server setup
│   ├── game.rs             # Game logic (Conway rules)
│   └── api.rs              # HTTP handlers
├── frontend/               # React app
│   ├── package.json
│   ├── vite.config.ts
│   ├── src/
│   │   ├── App.tsx
│   │   ├── components/
│   │   │   ├── GridCanvas.tsx
│   │   │   ├── ControlPanel.tsx
│   │   │   └── StatsDisplay.tsx
│   │   ├── hooks/
│   │   │   └── useGameState.ts
│   │   └── index.css
│   └── index.html
└── SPEC.md
```
