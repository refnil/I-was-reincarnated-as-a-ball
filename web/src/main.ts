import './style.css'

import mGBA from '@thenick775/mgba-wasm';
import gameUrl from './game.gba?url';

const canvas = document.querySelector<HTMLCanvasElement>('#canvas')

if (canvas === null) {
    throw new Error("No canvas")
}

async function readROM() {
    const response = await fetch(gameUrl)
    return response.blob();
}

const rom = readROM();
const emulator = mGBA({canvas}).then(async emu => {await emu.FSInit(); return emu});

Promise.all([emulator, rom]).then(async ([emulator, rom]) => {
    await new Promise<void>(resolve => emulator.uploadRom(new File([rom], "game.gba"), resolve))

    await emulator.FSSync();

    if (!emulator.loadGame('/data/games/game.gba')) {
        throw Error("Could not load game")
    }
}).catch(console.error)

