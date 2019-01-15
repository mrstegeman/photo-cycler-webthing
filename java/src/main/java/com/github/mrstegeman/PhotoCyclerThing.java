package com.github.mrstegeman;

import org.json.JSONArray;
import org.json.JSONObject;
import org.mozilla.iot.webthing.Property;
import org.mozilla.iot.webthing.Thing;
import org.mozilla.iot.webthing.Value;
import org.mozilla.iot.webthing.WebThingServer;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.util.Arrays;
import java.util.Random;
import java.util.Timer;
import java.util.TimerTask;

import fi.iki.elonen.router.RouterNanoHTTPD;

/**
 * MPD client represented as a Web Thing.
 */
public class PhotoCyclerThing extends Thing {
    private String photosPath;
    private String staticPath;
    private double updateRate;
    private Value updateRateValue;
    private Value imageValue;
    private Timer timer;

    /**
     * Initialize the thing.
     */
    public PhotoCyclerThing(String photosPath, String staticPath) {
        super("Photo Cycler", new JSONArray(), "Photo Cycler");

        this.photosPath = photosPath;
        this.staticPath = staticPath;
        this.updateRate = 5;

        JSONObject updateRateMetadata = new JSONObject();
        updateRateMetadata.put("type", "number");
        updateRateMetadata.put("description", "Photo cycle rate");
        updateRateMetadata.put("minimum", 0);
        updateRateMetadata.put("unit", "second");
        updateRateMetadata.put("label", "Update Rate");
        this.updateRateValue =
                new Value(this.updateRate, v -> this.setUpdateRate((double)v));
        this.addProperty(new Property(this,
                                      "updateRate",
                                      this.updateRateValue,
                                      updateRateMetadata));

        JSONObject imageMetadata = new JSONObject();
        imageMetadata.put("@type", "ImageProperty");
        imageMetadata.put("type", "null");
        imageMetadata.put("description", "Current image");
        imageMetadata.put("label", "Image");
        imageMetadata.put("readOnly", true);
        JSONArray imageLinks = new JSONArray();
        JSONObject imageLink = new JSONObject();
        imageLink.put("rel", "alternate");
        imageLink.put("href", "/static/current.jpg");
        imageLink.put("mediaType", "image/jpeg");
        imageLinks.put(imageLink);
        imageMetadata.put("links", imageLinks);
        this.imageValue = new Value(null);
        this.addProperty(new Property(this,
                                      "image",
                                      this.imageValue,
                                      imageMetadata));

        this.timer = null;
        this.setUpdateRate(this.updateRate);
    }

    private class CycleTask extends TimerTask {
        private String photosPath;
        private String staticPath;

        public CycleTask(String photosPath, String staticPath) {
            this.photosPath = photosPath;
            this.staticPath = staticPath;
        }

        public void run() {
            File photos = new File(this.photosPath);
            File[] files = Arrays.stream(photos.listFiles()).filter(f -> {
                String path = f.getPath();
                int extensionIndex = path.lastIndexOf('.');
                if (extensionIndex > 0) {
                    String extension =
                            path.substring(extensionIndex + 1).toLowerCase();
                    if (extension.equals("jpg") || extension.equals("jpeg")) {
                        return true;
                    }
                }

                return false;
            }).toArray(File[]::new);

            Random random = new Random();
            File link = new File(this.staticPath, "current.jpg");
            File image = files[random.nextInt(files.length)];

            if (link.exists()) {
                link.delete();
            }

            try {
                Files.createSymbolicLink(link.toPath(), image.toPath());
            } catch (IOException e) {
                System.err.println(e);
            }
        }
    }

    private void setUpdateRate(double value) {
        if (this.timer != null) {
            this.timer.cancel();
        }

        this.updateRate = value;
        this.timer = new Timer();
        this.timer.scheduleAtFixedRate(new CycleTask(this.photosPath,
                                                     this.staticPath),
                                       (long)this.updateRate * 1000,
                                       (long)this.updateRate * 1000);
    }

    /**
     * Create our MPD Web Thing and run the server.
     */
    public static void main(String[] args) {
        if (args.length < 2) {
            System.err.println(
                    "Usage: java -jar <jar_name> <photos_path> <static_path>");
            System.exit(1);
        }

        File photosPath = new File(args[0]);
        if (!photosPath.exists()) {
            System.err.println("Photos directory does not exist");
            System.exit(1);
        }

        File staticPath = new File(args[1]);
        if (!staticPath.exists()) {
            System.err.println("Static directory does not exist");
            System.exit(1);
        }

        try {
            WebThingServer.Route route = new WebThingServer.Route("/static/(.)+",
                                                                  RouterNanoHTTPD.StaticPageHandler.class,
                                                                  new Object[]{
                                                                          staticPath
                                                                  });
            PhotoCyclerThing thing =
                    new PhotoCyclerThing(photosPath.getCanonicalPath(),
                                         staticPath.getCanonicalPath());

            WebThingServer server =
                    new WebThingServer(new WebThingServer.SingleThing(thing),
                                       8888,
                                       null,
                                       null,
                                       Arrays.asList(route));

            Runtime.getRuntime()
                   .addShutdownHook(new Thread(() -> server.stop()));

            server.start(false);
        } catch (IOException e) {
            System.out.println(e);
            System.exit(1);
        }
    }
}