# photo-cycler-webthing

Simple web thing that cycles through a photo directory, updating the current image periodically.

This was created as an example of how to set up custom routes through the server and to create a custom UI for your web thing.

## Running

Drop a bunch of pictures (.jpg) in the photos folder, pick your desired language (python, node, java, rust), and run the `run.sh` script inside that directory.

## Installation

1. Clone this git repo to your Raspberry Pi (or some other computer) running the Mozilla Gateway.

    `git clone https://github.com/mrstegeman/photo-cycler-webthing.git`

2. Drop some pictures (.jpg) in the photos directory.
3. `cd` into either python, node, rust, or java, then `./run.sh`, e.g.:

    ```bash
    cd python
    ./run.sh
    ```

4. Ensure that thing-url-adapter, aka the "Web Thing" add-on, is installed on your gateway.
5. Click + on the things screen on your gateway's UI.
6. If your "Photo Cycler" thing shows up, add it. If not, you can click "Add by URL" and follow the prompts. You'll use something like `http://127.0.0.1:8888/` or `http://<hostname>.local:8888/`.
7. After adding your new thing, a little link icon will show up at the top left of its icon on your Things screen. Clicking that will yield a "picture frame".
