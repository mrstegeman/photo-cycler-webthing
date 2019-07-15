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

/**
 * Photo cycler web thing.
 */
class PhotoCyclerThing extends Thing {
  /**
   * Initialize the thing.
   */
  constructor(photosPath, staticPath) {
    super('urn:dev:ops:photo-cycler', 'Photo Cycler', [], 'Photo Cycler');

    this.photosPath = photosPath;
    this.staticPath = staticPath;
    this.updateRate = 5;

    this.addProperty(
      new Property(
        this,
        'updateRate',
        new Value(this.updateRate, this.setUpdateRate.bind(this)),
        {
          type: 'integer',
          description: 'Photo cycle rate',
          minimum: 0,
          unit: 'second',
          title: 'Update Rate',
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
          title: 'Image',
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

    this.setUiHref('/static/index.html');

    this.timer = null;
    this.setUpdateRate(this.updateRate);
  }

  /**
   * Set the new update rate and update the timer.
   *
   * @param {number} value New update rate
   */
  setUpdateRate(value) {
    if (this.timer) {
      clearInterval(this.timer);
    }

    this.updateRate = value;
    this.timer = setInterval(
      this.cycleImage.bind(this),
      this.updateRate * 1000
    );
  }

  /**
   * Update the current image.
   */
  cycleImage() {
    const files = fs.readdirSync(this.photosPath).filter((p) => {
      return mime.lookup(path.join(this.photosPath, p)) === 'image/jpeg';
    });

    if (files.length === 0) {
      return;
    }

    const linkPath = path.join(this.staticPath, 'current.jpg');
    const image = files[Math.floor(Math.random() * files.length)];

    try {
      if (fs.existsSync(linkPath)) {
        fs.unlinkSync(linkPath);
      }

      fs.symlinkSync(path.join(this.photosPath, image), linkPath);
    } catch (e) {
      console.error(e);
    }
  }
}

/**
 * Create our MPD Web Thing and run the server.
 */
function runServer(photosPath, staticPath) {
  if (!fs.existsSync(photosPath)) {
    console.error('Photos directory does not exist');
    process.exit(1);
  }

  if (!fs.existsSync(staticPath)) {
    console.error('Static directory does not exist');
    process.exit(1);
  }

  const thing = new PhotoCyclerThing(photosPath, staticPath);
  const server = new WebThingServer(
    new SingleThing(thing),
    8888,
    null,
    null,
    [
      {
        path: '/static',
        handler: express.static(staticPath),
      },
    ]
  );

  process.on('SIGINT', () => {
    server.stop();
    process.exit();
  });

  server.start();
}

if (process.argv.length < 4) {
  console.error(`Usage: node ${process.argv[0]} <photos_path> <static_path>`);
  process.exit(1);
}

runServer(path.normalize(process.argv[2]), path.normalize(process.argv[3]));
