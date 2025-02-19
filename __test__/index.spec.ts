import test from 'ava'

import { Encoder, Frame, DecodeOptions, ColorOutput } from '../index'
import { readFileSync, writeFileSync } from 'fs'
import { createCanvas } from '@napi-rs/canvas'

test('encoder with canvas', (t) => {
  const canvas = createCanvas(1024, 1024)
  const ctx = canvas.getContext('2d')
  let gif = new Encoder(1024, 1024)
  gif.setRepeat(-1)

  function toFrame() {
    const { width, height } = canvas
    let frame = Frame.fromRgba(width, height, canvas.data())
    return frame
  }

  ctx.fillStyle = '#FF0000'
  ctx.fillRect(0, 0, 512, 512)
  ctx.fillStyle = '#FFFF00'
  ctx.fillRect(0, 0, 256, 256)
  gif.addFrame(toFrame())

  ctx.fillStyle = '#00FF00'
  ctx.fillRect(0, 0, 512, 512)
  gif.addFrame(toFrame())

  ctx.fillStyle = '#0000FF'
  ctx.fillRect(0, 0, 512, 512)
  gif.addFrame(toFrame())

  const buffer = gif.getBuffer()
  writeFileSync('./__test__/encoderoutput.gif', buffer)
  t.assert('nyaa')
})

test('decoder with options', (t) => {
  let options = new DecodeOptions()
  options.setColorOutput(ColorOutput.IndexedPixels)
  options.setMemoryLimit(-1)

  let gif = options.readInfo(readFileSync('./__test__/encoderoutput.gif'))
  console.log(
    `Width: ${gif.width}\n`,
    `Height: ${gif.height}\n`,
    `Loops: ${gif.loops}\n`,
    (() => {
      const frames = []
      let f

      while ((f = gif.readNextFrame())) {
        frames.push(f)
      }

      return frames
    })()
      .map(
        (f, i) =>
          `Frame: ${i}\n` +
          ` - Width: ${f.width}\n` +
          ` - Height: ${f.height}\n` +
          ` - Delay: ${f.delay}0ms\n` +
          ` - Palette: ${Array.from(f.palette ?? gif.globalPalette ?? [])}\n` +
          ` - Interlaced: ${f.interlaced}\n` +
          ` - Needs User Input: ${f.needsUserInput}\n` +
          ` - Transparent: ${f.transparent}\n` +
          ` - Top: ${f.top}\n` +
          ` - Left: ${f.left}\n` +
          ` - Dispose: ${f.dispose}\n` +
          ` - Buffer: ${Array.from(f.buffer).slice(0, 50)} (50 first)`,
      )
      .join('\n'),
  )
  t.assert('O_o')
})

test('encoder with indexed pixels', (t) => {
  let frames = [
    [0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0, 0, 0],
  ]

  let gif = new Encoder(6, 6, Uint8Array.from([0xff, 0, 0, 0, 0xff, 0]))
  gif.setRepeat(-1)

  frames.forEach((x) => {
    let f = Frame.fromIndexedPixels(gif.width, gif.height, Uint8Array.from(x))
    gif.addFrame(f)
  })

  const buffer = gif.getBuffer()
  writeFileSync('./__test__/encoderoutput2.gif', buffer)
  t.assert('mrawww')
})