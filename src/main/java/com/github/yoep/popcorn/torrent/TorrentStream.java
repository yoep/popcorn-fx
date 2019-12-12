package com.github.yoep.popcorn.torrent;

import com.frostwire.jlibtorrent.SessionManager;
import com.frostwire.jlibtorrent.SessionParams;
import com.frostwire.jlibtorrent.SettingsPack;
import com.frostwire.jlibtorrent.TorrentInfo;
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

@Slf4j
@Component
@RequiredArgsConstructor
public class TorrentStream {
    private final TaskExecutor taskExecutor;

    private SessionManager torrentSession;
    private boolean initialized;

    public void startStream(String torrentUrl) {
        if (!initialized)
            throw new TorrentException("TorrentStream is not yet initialized");

        TorrentInfo torrentInfo = getTorrentInfo(torrentUrl);
    }

    @PostConstruct
    private void init() {
        taskExecutor.execute(() -> {
            log.trace("Initializing TorrentStream...");
            initNativeLibraries();
            initSessionManager();
            initSettings();
            this.initialized = true;
            log.debug("TorrentStream initialized");
        });
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

    private void initSessionManager() {
        this.torrentSession = new SessionManager();
    }

    private void initSettings() {
        SettingsPack settingsPack = (new SettingsPack())
                .anonymousMode(true)
                .connectionsLimit(200)
                .downloadRateLimit(0)
                .uploadRateLimit(1)
                .sendBufferWatermark(16)
                .activeDhtLimit(88);

        if (!this.torrentSession.isRunning()) {
            this.torrentSession.start(new SessionParams(settingsPack));
        } else {
            this.torrentSession.applySettings(settingsPack);
        }
    }

    private TorrentInfo getTorrentInfo(String torrentUrl) throws TorrentInfoException {
        if (torrentUrl.startsWith("magnet")) {
            byte[] data = this.torrentSession.fetchMagnet(torrentUrl, 20);
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
