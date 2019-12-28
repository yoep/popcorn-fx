package com.github.yoep.popcorn.torrent;

import com.frostwire.jlibtorrent.*;
import com.github.yoep.popcorn.PopcornTimeApplication;
import com.github.yoep.popcorn.settings.SettingsService;
import com.github.yoep.popcorn.settings.models.TorrentSettings;
import com.github.yoep.popcorn.torrent.listeners.TorrentListener;
import com.github.yoep.popcorn.torrent.listeners.TorrentListenerHolder;
import com.github.yoep.popcorn.torrent.models.Torrent;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FileUtils;
import org.apache.commons.io.IOUtils;
import org.apache.commons.lang3.ArrayUtils;
import org.springframework.core.io.Resource;
import org.springframework.core.io.support.PathMatchingResourcePatternResolver;
import org.springframework.core.task.TaskExecutor;
import org.springframework.stereotype.Component;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import java.io.File;
import java.io.FileInputStream;
import java.io.IOException;
import java.io.InputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.util.Arrays;
import java.util.Optional;

@Slf4j
@Component
@RequiredArgsConstructor
public class TorrentStream {
    private static final String JNI_PATH = "jlibtorrent.jni.path";

    private final TorrentListenerHolder listenerHolder = new TorrentListenerHolder();
    private final TaskExecutor taskExecutor;
    private final SettingsService settingsService;

    private SessionManager torrentSession;
    private TorrentFactory torrentFactory;
    private Thread currentStream;
    private boolean initialized;

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
        return currentStream != null && currentStream.isAlive();
    }

    //endregion

    public void addListener(TorrentListener listener) {
        listenerHolder.addListener(listener);
    }

    public void removeListener(TorrentListener listener) {
        listenerHolder.removeListener(listener);
    }

    public void startStream(String torrentUrl) {
        if (!initialized)
            throw new TorrentException("TorrentStream is not yet initialized");

        if (isStreaming())
            stopStream();

        this.currentStream = new Thread(() -> {
            try {
                log.debug("DHT contains {} nodes", torrentSession.stats().dhtNodes());
                getTorrentInfo(torrentUrl)
                        .ifPresentOrElse(torrentInfo -> {
                            Priority[] priorities = new Priority[torrentInfo.numFiles()];

                            Arrays.fill(priorities, Priority.IGNORE);

                            this.torrentSession.download(torrentInfo, getSettings().getDirectory(), null, priorities, null);
                        }, () -> listenerHolder.onLoadError("Failed to retrieve torrent information"));
            } catch (Exception ex) {
                log.error(ex.getMessage(), ex);
            }
        });

        // start a new thread
        taskExecutor.execute(this.currentStream);
    }

    public void stopStream() {
        if (this.torrentFactory == null || this.torrentFactory.getCurrentTorrent().map(e -> e.getState() == Torrent.State.PAUSED).orElse(true))
            return;

        log.debug("Stopping current torrent stream");

        if (this.currentStream != null)
            this.currentStream.interrupt();
        if (this.torrentFactory != null) {
            this.torrentFactory.getCurrentTorrent().ifPresent(torrent -> {
                torrent.pause();
                this.torrentSession.remove(torrent.getTorrentHandle());
            });
        }
    }

    @PostConstruct
    private void init() {
        taskExecutor.execute(() -> {
            log.trace("Initializing TorrentStream");
            long startTime = System.currentTimeMillis();
            initNativeLibraries();
            initSession();
            initSettings();
            initSaveDirectory();
            initDHT(startTime);
        });
    }

    @PreDestroy
    public void destroy() {
        stopStream();

        if (this.torrentSession != null)
            this.torrentSession.stop();
    }

    private void initNativeLibraries() {
        log.trace("Initializing native libraries for jlibtorrent");
        try {
            PathMatchingResourcePatternResolver resourcePatternResolver = new PathMatchingResourcePatternResolver(this.getClass().getClassLoader());
            Resource[] resources = resourcePatternResolver.getResources("classpath*:**/*jlibtorrent-*");

            if (ArrayUtils.isNotEmpty(resources)) {
                Resource resource = resources[0];
                String filename = resource.getFilename();
                File destination = new File(PopcornTimeApplication.APP_DIR + File.separator + filename);
                String absolutePath = destination.getAbsolutePath();

                if (!destination.exists()) {
                    log.trace("Copying resource {} to {}", filename, absolutePath);
                    FileUtils.copyInputStreamToFile(resource.getInputStream(), destination);
                }

                log.trace("Updating system property \"{}\" to \"{}\"", JNI_PATH, absolutePath);
                System.setProperty(JNI_PATH, absolutePath);
            } else {
                log.error("Unable to find any jlibtorrent resources in the classpath");
            }
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    private void initSession() {
        this.torrentSession = new SessionManager();
        this.torrentFactory = new TorrentFactory(this.torrentSession, this.listenerHolder);
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

    private void initSaveDirectory() {
        File directory = getSettings().getDirectory();

        if (directory.mkdirs()) {
            log.info("Created torrent save directory in {}", directory.getAbsolutePath());
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
                log.warn("Unexpectedly quit of DHT monitor", e);
            }

            this.initialized = true;
            log.info("TorrentStream initialized in {} seconds", (System.currentTimeMillis() - startTime) / 1000.0);
        });
    }

    /**
     * Get torrent metadata, either by downloading the .torrent or fetching the magnet
     *
     * @param torrentUrl {@link String} URL to .torrent or magnet link
     * @return {@link TorrentInfo}
     */
    private Optional<TorrentInfo> getTorrentInfo(String torrentUrl) {
        if (torrentUrl.startsWith("magnet")) {
            byte[] data = torrentSession.fetchMagnet(torrentUrl, 60);
            if (data != null)
                try {
                    return Optional.of(TorrentInfo.bdecode(data));
                } catch (IllegalArgumentException e) {
                    throw new TorrentException("No torrent info could be found or read", e);
                }

        } else if (torrentUrl.startsWith("http") || torrentUrl.startsWith("https")) {
            try {
                URL url = new URL(torrentUrl);
                HttpURLConnection connection = (HttpURLConnection) url.openConnection();

                connection.setRequestMethod("GET");
                connection.setInstanceFollowRedirects(true);
                connection.connect();

                InputStream inputStream = connection.getInputStream();

                byte[] responseByteArray = new byte[0];

                if (connection.getResponseCode() == 200) {
                    responseByteArray = IOUtils.toByteArray(inputStream);
                }

                inputStream.close();
                connection.disconnect();

                if (responseByteArray.length > 0) {
                    return Optional.of(TorrentInfo.bdecode(responseByteArray));
                }
            } catch (IOException | IllegalArgumentException ex) {
                throw new TorrentException("No torrent info could be found or read", ex);
            }
        } else if (torrentUrl.startsWith("file")) {
            File file = new File(torrentUrl);

            try {
                FileInputStream fileInputStream = new FileInputStream(file);
                byte[] responseByteArray = IOUtils.toByteArray(fileInputStream);
                fileInputStream.close();

                if (responseByteArray.length > 0) {
                    return Optional.of(TorrentInfo.bdecode(responseByteArray));
                }
            } catch (IOException | IllegalArgumentException ex) {
                throw new TorrentException("No torrent info could be found or read", ex);
            }
        }

        return Optional.empty();
    }

    private TorrentSettings getSettings() {
        return settingsService.getSettings().getTorrentSettings();
    }
}
