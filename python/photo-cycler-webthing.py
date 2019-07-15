"""Web thing which cycles through a list of photos."""

from webthing import SingleThing, Property, Thing, Value, WebThingServer
import mimetypes
import os
import random
import sys
import tornado.ioloop
import tornado.web


class PhotoCyclerThing(Thing):
    """Photo cycler web thing."""

    def __init__(self, photos_path, static_path):
        """Initialize the thing."""
        Thing.__init__(
            self,
            'urn:dev:ops:photo-cycler',
            'Photo Cycler',
            [],
            'Photo Cycler'
        )

        self.photos_path = photos_path
        self.static_path = static_path
        self.update_rate = 5

        self.add_property(
            Property(self,
                     'updateRate',
                     Value(self.update_rate, self.set_update_rate),
                     metadata={
                         'type': 'integer',
                         'description': 'Photo cycle rate',
                         'minimum': 0,
                         'unit': 'second',
                         'title': 'Update Rate',
                     }))

        self.add_property(
            Property(self,
                     'image',
                     Value(None),
                     metadata={
                         '@type': 'ImageProperty',
                         'type': 'null',
                         'description': 'Current image',
                         'title': 'Image',
                         'readOnly': True,
                         'links': [
                             {
                                 'rel': 'alternate',
                                 'href': '/static/current.jpg',
                                 'mediaType': 'image/jpeg',
                             },
                         ],
                     }))

        self.set_ui_href('/static/index.html')

        self.timer = None
        self.set_update_rate(self.update_rate)

    def set_update_rate(self, value):
        """
        Set the new update rate and update the timer.

        value -- new update rate
        """
        if self.timer is not None:
            self.timer.stop()

        self.update_rate = value
        self.timer = tornado.ioloop.PeriodicCallback(
            self.cycle_image,
            self.update_rate * 1000
        )
        self.timer.start()

    def cycle_image(self):
        """Update the current image."""
        files = [
            p for p in os.listdir(self.photos_path)
            if mimetypes.guess_type(
                os.path.join(self.photos_path, p)
            )[0] == 'image/jpeg'
        ]

        if len(files) == 0:
            return

        try:
            link_path = os.path.join(self.static_path, 'current.jpg')

            if os.path.exists(link_path):
                os.unlink(link_path)

            os.symlink(os.path.join(self.photos_path, random.choice(files)),
                       link_path)
        except OSError as e:
            print(e)


def run_server(photos_path, static_path):
    """Create our photo cycler web thing and run the server."""
    if not os.path.isdir(photos_path):
        print('Photos directory does not exist')
        sys.exit(1)

    if not os.path.isdir(static_path):
        print('Static directory does not exist')
        sys.exit(1)

    thing = PhotoCyclerThing(photos_path, static_path)
    server = WebThingServer(
        SingleThing(thing),
        port=8888,
        additional_routes=[
            (
                r'/static/(.*)',
                tornado.web.StaticFileHandler,
                {'path': static_path},
            ),
        ]
    )

    try:
        server.start()
    except KeyboardInterrupt:
        server.stop()


if __name__ == '__main__':
    if len(sys.argv) < 3:
        print('Usage: {} <photos_path> <static_path>'.format(sys.argv[0]))
        sys.exit(1)

    run_server(os.path.realpath(sys.argv[1]), os.path.realpath(sys.argv[2]))
