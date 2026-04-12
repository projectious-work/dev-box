# PixiJS API Cheatsheet

## Application Setup

```js
import { Application, Assets, Sprite, Container, Graphics, Text } from 'pixi.js';

const app = new Application();
await app.init({
  width: 800,
  height: 600,
  background: '#1a1a2e',
  antialias: true,
  resolution: window.devicePixelRatio || 1,
  autoDensity: true,
});
document.body.appendChild(app.canvas);
```

## Core Classes

| Class | Purpose | Key Properties |
|---|---|---|
| `Application` | Entry point, manages renderer and ticker | `stage`, `ticker`, `screen`, `canvas` |
| `Sprite` | Displays a texture | `texture`, `anchor`, `tint`, `alpha` |
| `Container` | Groups display objects | `children`, `sortableChildren`, `zIndex` |
| `Graphics` | Vector drawing | `rect()`, `circle()`, `fill()`, `stroke()` |
| `Text` | Renders text | `text`, `style`, `resolution` |
| `AnimatedSprite` | Frame-based animation | `textures`, `animationSpeed`, `currentFrame` |
| `ParticleContainer` | High-performance sprite batch | `maxSize`, `properties` |
| `Ticker` | Game loop manager | `deltaTime`, `FPS`, `speed` |

## Asset Loading

```js
// Single asset
const texture = await Assets.load('hero.png');
const sprite = new Sprite(texture);

// Bundled assets
Assets.addBundle('level1', {
  background: 'bg.png',
  tileset: 'tiles.json',
  music: 'theme.mp3',
});
await Assets.loadBundle('level1');

// With progress tracking
Assets.loadBundle('level1', (progress) => {
  console.log(`Loading: ${Math.round(progress * 100)}%`);
});
```

## Sprite Operations

```js
const sprite = Sprite.from(texture);
sprite.anchor.set(0.5);           // Center origin
sprite.position.set(400, 300);    // Set position
sprite.scale.set(2);              // Uniform scale
sprite.rotation = Math.PI / 4;    // Rotate 45 degrees
sprite.alpha = 0.8;               // Semi-transparent
sprite.tint = 0xff6b6b;           // Color tint
sprite.eventMode = 'static';      // Enable interaction
sprite.cursor = 'pointer';        // Pointer cursor
```

## Game Loop (Ticker)

```js
app.ticker.add((ticker) => {
  // ticker.deltaTime = 1.0 at 60fps, scales with frame time
  sprite.x += speed * ticker.deltaTime;
  sprite.rotation += 0.01 * ticker.deltaTime;
});

// Control ticker
app.ticker.speed = 0.5;   // Half speed
app.ticker.stop();         // Pause
app.ticker.start();        // Resume
```

## Event Handling

```js
sprite.eventMode = 'static';
sprite.on('pointerdown', (event) => {
  console.log('Clicked at', event.global.x, event.global.y);
});
sprite.on('pointerover', () => { sprite.tint = 0xffff00; });
sprite.on('pointerout', () => { sprite.tint = 0xffffff; });

// Drag pattern
let dragging = false;
sprite.on('pointerdown', () => { dragging = true; });
app.stage.on('pointermove', (e) => {
  if (dragging) sprite.position.copyFrom(e.global);
});
app.stage.on('pointerup', () => { dragging = false; });
app.stage.eventMode = 'static';
app.stage.hitArea = app.screen;
```

## Graphics Drawing

```js
const g = new Graphics();
// Rectangle
g.rect(0, 0, 200, 100).fill({ color: 0x3498db, alpha: 0.8 });
// Circle
g.circle(100, 100, 50).fill(0xe74c3c).stroke({ color: 0xffffff, width: 2 });
// Line
g.moveTo(0, 0).lineTo(200, 200).stroke({ color: 0x2ecc71, width: 3 });
// Rounded rectangle
g.roundRect(10, 10, 180, 80, 12).fill(0x9b59b6);
```

## Text Rendering

```js
const text = new Text({
  text: 'Score: 0',
  style: {
    fontFamily: 'Arial',
    fontSize: 36,
    fontWeight: 'bold',
    fill: 0xffffff,
    stroke: { color: 0x000000, width: 4 },
    dropShadow: { color: 0x000000, blur: 4, distance: 2 },
  },
});
text.anchor.set(0.5);
```

## AnimatedSprite

```js
import { AnimatedSprite, Spritesheet } from 'pixi.js';

// From spritesheet
const sheet = await Assets.load('character.json');
const walkFrames = sheet.animations['walk']; // array of textures
const character = new AnimatedSprite(walkFrames);
character.animationSpeed = 0.15;
character.play();

// Control
character.stop();
character.gotoAndPlay(0);
character.loop = true;
character.onComplete = () => { /* animation finished */ };
```

## Filters

```js
import { BlurFilter, ColorMatrixFilter } from 'pixi.js';

sprite.filters = [new BlurFilter({ strength: 4 })];

const colorMatrix = new ColorMatrixFilter();
colorMatrix.brightness(1.5);
colorMatrix.saturate(0.5);
sprite.filters = [colorMatrix];

// Remove filters
sprite.filters = [];
```

## Common Patterns

### Responsive resize
```js
const app = new Application();
await app.init({ resizeTo: window });

window.addEventListener('resize', () => {
  // Reposition UI elements relative to screen
  scoreText.x = app.screen.width - 20;
  player.x = app.screen.width / 2;
});
```

### Object pooling
```js
const pool = [];
function getSprite(texture) {
  const sprite = pool.pop() || new Sprite(texture);
  sprite.visible = true;
  return sprite;
}
function releaseSprite(sprite) {
  sprite.visible = false;
  pool.push(sprite);
}
```

### Scene management
```js
const scenes = { menu: new Container(), game: new Container() };
function switchScene(name) {
  Object.values(scenes).forEach(s => s.visible = false);
  scenes[name].visible = true;
}
app.stage.addChild(scenes.menu, scenes.game);
switchScene('menu');
```
