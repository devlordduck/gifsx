# `gifsx`

A `gif` rust crate binding for Node.js.

## Install
```sh
yarn add @gifsx/gifsx
npm install @gifsx/gifsx
bun install @gifsx/gifsx
```

## Examples

### Encoding a GIF

```javascript
const { Encoder, Frame } = require('@gifsx/gifsx')
const { createCanvas } = require('@napi-rs/canvas')
const fs = require('fs')

// Create a canvas
const canvas = createCanvas(512, 512)
const { width, height } = canvas
const ctx = canvas.getContext('2d')

// Create a new encoder
const encoder = new Encoder(512, 512)
encoder.setRepeat(-1)

// Function to convert canvas to frame
function toFrame() {
  return Frame.fromRgba(width, height, ctx.getImageData(0, 0, width, height).data)
}

// Draw and add frames
ctx.fillStyle = '#FF0000'
ctx.fillRect(0, 0, 512, 512)
encoder.addFrame(toFrame())

ctx.fillStyle = '#00FF00'
ctx.fillRect(0, 0, 512, 512)
encoder.addFrame(toFrame())

ctx.fillStyle = '#0000FF'
ctx.fillRect(0, 0, 512, 512)
encoder.addFrame(toFrame())

// Get the encoded GIF buffer
const buffer = encoder.getBuffer()
fs.writeFileSync('./output.gif', buffer)
```

### Decoding a GIF with config

```javascript
const { DecodeOptions, ColorOutput } = require('@gifsx/gifsx')
const fs = require('fs')

// Create decode options
const options = new DecodeOptions()
options.setColorOutput(ColorOutput.Rgba)
options.setMemoryLimit(-1)

// Decode the GIF
const gif = options.readInfo(fs.readFileSync('./input.gif'))
console.log(`Width: ${gif.width}, Height: ${gif.height}, Loops: ${gif.loops}`)

// Read frames
let frame
while ((frame = gif.readNextFrame())) {
  console.log(`Frame: Width: ${frame.width}, Height: ${frame.height}, Delay: ${frame.delay}ms`)
}
```

### Decoding a GIF without config

```javascript
const { Decoder } = require('@gifsx/gifsx')
const fs = require('fs')

// Decode the GIF
const gif = new Decoder(fs.readFileSync('./input.gif'));
console.log(`Width: ${gif.width}, Height: ${gif.height}, Loops: ${gif.loops}`)

// Read frames
let frame
while ((frame = gif.readNextFrame())) {
  console.log(`Frame: Width: ${frame.width}, Height: ${frame.height}, Delay: ${frame.delay}ms`)
}
```