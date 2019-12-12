package com.github.yoep.popcorn.torrent;

import com.frostwire.jlibtorrent.*;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FileUtils;
import org.apache.commons.io.IOUtils;
import org.springframework.core.io.Resource;
import org.springframework.core.io.support.PathMatchingResourcePatternResolver;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;

import javax.annotation.PostConstruct;
import java.io.File;
import java.io.FileInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.util.Arrays;

@Slf4j
@Component
@RequiredArgsConstructor
public class TorrentStream {
    private static final String SAVE_DIRECTORY_NAME = ".popcorn-time";
    private final TaskExecutor taskExecutor;

    private SessionManager torrentSession;
    private boolean initialized;
    private boolean streaming;

    //region Getters & Setters

    /**
     * Get if this torrent stream instance is initialized.
     *
     * @return Returns true if this instance is initialized and ready for use, else false.
     */
    public boolean isInitialized() {
        return initialized;
    }

    /**
     * Get if this instance is streaming.
     *
     * @return Returns true if this instance is streaming, else false.
     */
    public boolean isStreaming() {
        return streaming;
    }

    //endregion

    public void startStream(String torrentUrl) {
        if (!initialized)
            throw new TorrentException("TorrentStream is not yet initialized");

        // start a new thread
        taskExecutor.execute(() -> {
            try {
                log.debug("DHT contains {} nodes", torrentSession.stats().dhtNodes());
                TorrentInfo torrentInfo = getTorrentInfo(torrentUrl);

                Priority[] priorities = new Priority[torrentInfo.numFiles()];

                Arrays.fill(priorities, Priority.IGNORE);

                this.torrentSession.download(torrentInfo, getSaveDirectory(), null, priorities, null);
                this.streaming = true;
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
            }
        });
    }

    public void stopStream() {
        if (!this.streaming)
            return;


    }

    @PostConstruct
    private void init() {
        taskExecutor.execute(() -> {
            log.trace("Initializing TorrentStream");
            long startTime = System.currentTimeMillis();
            initNativeLibraries();
            this.torrentSession = new SessionManager();
            initSettings();
            initSaveDirectory();
            initDHT(startTime);
        });
    }

    private void initSaveDirectory() {
        File directory = getSaveDirectory();

        if (directory.mkdirs()) {
            log.info("Created torrent save directory in {}", directory.getAbsolutePath());
        }
    }

    private void initNativeLibraries() {
        try {
            PathMatchingResourcePatternResolver resourcePatternResolver = new PathMatchingResourcePatternResolver(this.getClass().getClassLoader());
            Resource[] resources = resourcePatternResolver.getResources("classpath*:**/jlibtorrent-*");
            String workingDir = System.getProperty("user.dir");

            for (Resource resource : resources) {
                String filename = resource.getFilename();
                File destination = new File(workingDir + File.separator + filename);

                if (!destination.exists())
                    FileUtils.copyInputStreamToFile(resource.getInputStream(), destination);
            }
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    private void initSettings() {
        SettingsPack settingsPack = (new SettingsPack())
                .anonymousMode(true)
                .connectionsLimit(200)
                .downloadRateLimit(0)
                .uploadRateLimit(0)
                .sendBufferWatermark(16)
                .activeDhtLimit(88);

        if (!this.torrentSession.isRunning()) {
            this.torrentSession.start(new SessionParams(settingsPack));
        } else {
            this.torrentSession.applySettings(settingsPack);
        }
    }

    private void initDHT(long startTime) {
        log.trace("Starting torrent session DHT");
        this.torrentSession.startDht();

        taskExecutor.execute(() -> {
            try {
                // wait for the dht to have at least 10 nodes before continuing
                while (this.torrentSession.stats().dhtNodes() < 10) {
                    Thread.sleep(100);
                }
            } catch (InterruptedException e) {
                log.warn("Unexpectedly quit of DHT monitor with " + e.getMessage(), e);
            }

            this.initialized = true;
            log.info("TorrentStream initialized in {} seconds", (System.currentTimeMillis() - startTime) / 1000.0);
        });
    }

    private File getSaveDirectory() {
        return new File(System.getProperty("user.home") + File.separator + SAVE_DIRECTORY_NAME);
    }

    private TorrentInfo getTorrentInfo(String torrentUrl) throws TorrentInfoException {
        if (torrentUrl.startsWith("magnet")) {
            byte[] data = this.torrentSession.fetchMagnet(torrentUrl, 60);
            if (data != null) {
                try {
                    return TorrentInfo.bdecode(data);
                } catch (IllegalArgumentException var6) {
                    throw new TorrentInfoException(var6);
                }
            }
        } else {
            byte[] responseByteArray;
            if (!torrentUrl.startsWith("http") && !torrentUrl.startsWith("https")) {
                if (torrentUrl.startsWith("file")) {
                    File file = new File(torrentUrl);

                    try {
                        FileInputStream fileInputStream = new FileInputStream(file);
                        responseByteArray = IOUtils.toByteArray(fileInputStream);
                        fileInputStream.close();
                        if (responseByteArray.length > 0) {
                            return TorrentInfo.bdecode(responseByteArray);
                        }
                    } catch (IllegalArgumentException | IOException var7) {
                        throw new TorrentInfoException(var7);
                    }
                }
            } else {
                try {
                    URL url = new URL(torrentUrl);
                    HttpURLConnection connection = (HttpURLConnection) url.openConnection();
                    connection.setRequestMethod("GET");
                    connection.setInstanceFollowRedirects(true);
                    connection.connect();
                    InputStream inputStream = connection.getInputStream();
                    responseByteArray = new byte[0];
                    if (connection.getResponseCode() == 200) {
                        responseByteArray = IOUtils.toByteArray(inputStream);
                    }

                    inputStream.close();
                    connection.disconnect();
                    if (responseByteArray.length > 0) {
                        return TorrentInfo.bdecode(responseByteArray);
                    }
                } catch (IllegalArgumentException | IOException var8) {
                    throw new TorrentInfoException(var8);
                }
            }
        }

        return null;
    }
}
