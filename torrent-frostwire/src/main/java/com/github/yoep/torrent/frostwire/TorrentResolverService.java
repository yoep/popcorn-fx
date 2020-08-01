package com.github.yoep.torrent.frostwire;

import com.github.yoep.torrent.adapter.InvalidTorrentUrlException;
import com.github.yoep.torrent.adapter.TorrentException;
import com.github.yoep.torrent.frostwire.wrappers.TorrentInfoWrapper;
import lombok.RequiredArgsConstructor;
import org.apache.commons.io.IOUtils;
import org.apache.http.client.HttpClient;
import org.apache.http.client.methods.HttpGet;
import org.springframework.util.Assert;

import java.io.IOException;
import java.net.URI;
import java.net.URISyntaxException;
import java.net.URL;

@RequiredArgsConstructor
public class TorrentResolverService {
    private final TorrentSessionManager sessionManager;
    private final HttpClient httpClient;

    //region Methods

    /**
     * Resolve the given torrent url to a torrent info.
     *
     * @param torrentUrl The torrent url to resolve.
     * @return Returns the resolved torrent url info.
     */
    public TorrentInfoWrapper resolveUrl(String torrentUrl) {
        Assert.hasText(torrentUrl, "torrentUrl cannot be empty");

        if (isMagnetUrl(torrentUrl)) {
            return fetchMagnetInfo(torrentUrl);
        } else if (isHttpUrl(torrentUrl)) {
            return fetchHttpInfo(torrentUrl);
        } else if (isFileUrl(torrentUrl)) {
            return fetchFileInfo(torrentUrl);
        }

        throw new InvalidTorrentUrlException(torrentUrl);
    }

    //endregion

    //region Functions

    private TorrentInfoWrapper fetchMagnetInfo(String magnetUrl) {
        var session = sessionManager.getSession();
        var data = session.fetchMagnet(magnetUrl, 60);

        // check if the magnet data was fetched with success
        if (data == null)
            throw new TorrentException("Unable to fetch magnet torrent, no data available");

        try {
            var torrentInfo = com.frostwire.jlibtorrent.TorrentInfo.bdecode(data);

            return toTorrentInfoWrapper(torrentInfo);
        } catch (IllegalArgumentException ex) {
            throw new TorrentException("Unable to fetch magnet torrent, torrent info could not be found or read", ex);
        }
    }

    private TorrentInfoWrapper fetchHttpInfo(String torrentUrl) {
        try {
            var request = new HttpGet(new URI(torrentUrl));
            var response = httpClient.execute(request);
            var statusCode = response.getStatusLine().getStatusCode();

            if (statusCode == 200) {
                var responseData = response.getEntity().getContent();
                var responseBytes = IOUtils.toByteArray(responseData);

                if (responseBytes.length > 0) {
                    var torrentInfo = com.frostwire.jlibtorrent.TorrentInfo.bdecode(responseBytes);

                    return toTorrentInfoWrapper(torrentInfo);
                } else {
                    throw new TorrentException("Unable to fetch HTTP torrent, no data available");
                }
            } else {
                throw new TorrentException("Unable to fetch HTTP torrent, invalid response status " + statusCode);
            }
        } catch (URISyntaxException ex) {
            throw new TorrentException("Unable to fetch HTTP torrent, the torrent url is invalid", ex);
        } catch (IOException ex) {
            throw new TorrentException("Unable to fetch HTTP torrent, torrent info could not be found or read", ex);
        }
    }

    private TorrentInfoWrapper fetchFileInfo(String url) {
        try {
            var fileBytes = IOUtils.toByteArray(new URL(url));

            if (fileBytes.length > 0) {
                var torrentInfo = com.frostwire.jlibtorrent.TorrentInfo.bdecode(fileBytes);

                return toTorrentInfoWrapper(torrentInfo);
            } else {
                throw new TorrentException("Unable to fetch file torrent, no data available");
            }
        } catch (IOException | IllegalArgumentException ex) {
            throw new TorrentException("Unable to fetch file torrent, torrent info could not be found or read", ex);
        }
    }

    private boolean isMagnetUrl(String url) {
        return url.startsWith("magnet");
    }

    private boolean isHttpUrl(String url) {
        return url.startsWith("http") || url.startsWith("https");
    }

    private boolean isFileUrl(String url) {
        return url.startsWith("file");
    }

    private TorrentInfoWrapper toTorrentInfoWrapper(com.frostwire.jlibtorrent.TorrentInfo info) {
        return new TorrentInfoWrapper(info);
    }

    //endregion
}
