/**
 * Web thing which cycles through a list of photos.
 */

const {
  Property,
  SingleThing,
  Thing,
  Value,
  WebThingServer,
} = require('webthing');
const express = require('express');
const fs = require('fs');
const mime = require('mime-types');
const path = require('path');

const _BASE_PATH = path.normalize(path.join(__dirname, '..'));
const _PHOTOS_PATH = path.join(_BASE_PATH, 'photos');
const _STATIC_PATH = path.join(_BASE_PATH, 'static');

/**
 * Photo cycler web thing.
 */
class PhotoCyclerThing extends Thing {
  /**
   * Initialize the thing.
   */
  constructor() {
    super('Photo Cycler', [], 'Photo Cycler');

    this.updateRate = 5;

    this.addProperty(
      new Property(
        this,
        'updateRate',
        new Value(this.updateRate, this.setUpdateRate.bind(this)),
        {
          type: 'number',
          description: 'Photo cycle rate',
          minimum: 0,
          unit: 'second',
          label: 'Update Rate',
        }
      )
    );

    this.addProperty(
      new Property(
        this,
        'image',
        new Value(null),
        {
          '@type': 'ImageProperty',
          type: 'null',
          description: 'Current image',
          label: 'Image',
          readOnly: true,
          links: [
            {
              rel: 'alternate',
              href: '/static/current.jpg',
              mediaType: 'image/jpeg',
            },
          ],
        }
      )
    );

    this.timer = setInterval(this.cycleImage, this.updateRate * 1000);
  }

  /**
   * Set the new update rate and update the timer.
   *
   * @param {number} value New update rate
   */
  setUpdateRate(value) {
    this.updateRate = value;
    clearInterval(this.timer);
    this.timer = setInterval(this.cycleImage, this.updateRate * 1000);
  }

  /**
   * Update the current image.
   */
  cycleImage() {
    const files = fs.readdirSync(_PHOTOS_PATH).filter((p) => {
      return mime.lookup(path.join(_PHOTOS_PATH, p)) === 'image/jpeg';
    });

    if (files.length === 0) {
      return;
    }

    const linkPath = path.join(_STATIC_PATH, 'current.jpg');
    const image = files[Math.floor(Math.random() * files.length)];

    try {
      if (fs.existsSync(linkPath)) {
        fs.unlinkSync(linkPath);
      }

      fs.symlinkSync(path.join(_PHOTOS_PATH, image), linkPath);
    } catch (e) {
      console.error(e);
    }
  }
}

/**
 * Create our MPD Web Thing and run the server.
 */
function runServer() {
  if (!fs.existsSync(_PHOTOS_PATH)) {
    try {
      fs.mkdirSync(_PHOTOS_PATH, 0o755);
    } catch (e) {
      console.error('Photos directory does not exists, failed to create:', e);
      process.exit(1);
    }
  }

  if (!fs.existsSync(_STATIC_PATH)) {
    try {
      fs.mkdirSync(_STATIC_PATH, 0o755);
    } catch (e) {
      console.error('Static directory does not exists, failed to create:', e);
      process.exit(1);
    }
  }

  const thing = new PhotoCyclerThing();
  const server = new WebThingServer(new SingleThing(thing), 8888);

  server.app.use('/static', express.static(_STATIC_PATH));

  process.on('SIGINT', () => {
    server.stop();
    process.exit();
  });

  server.start();
}

runServer();
