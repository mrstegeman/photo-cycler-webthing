"""Web thing which cycles through a list of photos."""

from webthing import SingleThing, Property, Thing, Value, WebThingServer
import mimetypes
import os
import random
import sys
import tornado.ioloop
import tornado.web

_BASE_PATH = os.path.dirname(os.path.realpath(__file__))
_PHOTOS_PATH = os.path.join(_BASE_PATH, 'photos')
_STATIC_PATH = os.path.join(_BASE_PATH, 'static')


class PhotoCyclerThing(Thing):
    """Photo cycler web thing."""

    def __init__(self):
        """Initialize the thing."""
        Thing.__init__(self, 'Photo Cycler', [], 'Photo Cycler')

        self.update_rate = 5

        self.add_property(
            Property(self,
                     'updateRate',
                     Value(self.update_rate, self.set_update_rate),
                     metadata={
                         'type': 'number',
                         'description': 'Photo cycle rate',
                         'minimum': 0,
                         'unit': 'second',
                         'label': 'Update Rate',
                     }))

        self.add_property(
            Property(self,
                     'image',
                     Value(None),
                     metadata={
                         '@type': 'ImageProperty',
                         'type': 'null',
                         'description': 'Current image',
                         'label': 'Image',
                         'readOnly': True,
                         'links': [
                             {
                                 'rel': 'alternate',
                                 'href': '/static/current.jpg',
                                 'mediaType': 'image/jpeg',
                             },
                         ],
                     }))

        self.timer = tornado.ioloop.PeriodicCallback(self.cycle_image,
                                                     self.update_rate * 1000)
        self.timer.start()

    def set_update_rate(self, value):
        """
        Set the new update rate and update the timer.

        value -- new update rate
        """
        self.update_rate = value
        self.timer.stop()
        self.timer = tornado.ioloop.PeriodicCallback(
            self.cycle_image,
            self.update_rate * 1000
        )
        self.timer.start()

    def cycle_image(self):
        """Update the current image."""
        files = [p for p in os.listdir(_PHOTOS_PATH)
                 if mimetypes.guess_type(p)[0] == 'image/jpeg']

        if len(files) == 0:
            return

        try:
            link_path = os.path.join(_STATIC_PATH, 'current.jpg')

            if os.path.exists(link_path):
                os.unlink(link_path)

            os.symlink(os.path.join(_PHOTOS_PATH, random.choice(files)),
                       link_path)
        except OSError as e:
            print(e)


def run_server():
    """Create our photo cycler web thing and run the server."""
    if not os.path.isdir(_PHOTOS_PATH):
        try:
            os.mkdir(_PHOTOS_PATH, 0o755)
        except OSError as e:
            print('Photos directory does not exist, failed to create:', e)
            sys.exit(1)

    if not os.path.isdir(_STATIC_PATH):
        try:
            os.mkdir(_STATIC_PATH, 0o755)
        except OSError as e:
            print('Static directory does not exist, failed to create:', e)
            sys.exit(1)

    thing = PhotoCyclerThing()
    server = WebThingServer(SingleThing(thing), port=8888)

    server.app.add_handlers(
        r'.*',
        [
            (
                r'/static/(.*)',
                tornado.web.StaticFileHandler,
                {'path': _STATIC_PATH},
            ),
        ]
    )

    try:
        server.start()
    except KeyboardInterrupt:
        server.stop()


if __name__ == '__main__':
    run_server()
