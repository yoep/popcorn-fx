package com.github.yoep.popcorn.view.services;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.activities.ActivityManager;
import com.github.yoep.popcorn.activities.ErrorNotificationActivity;
import com.github.yoep.popcorn.activities.LoadUrlActivity;
import com.github.yoep.popcorn.activities.PlayVideoActivity;
import com.github.yoep.popcorn.messages.MediaMessage;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.util.regex.Pattern;

@Slf4j
@Service
@RequiredArgsConstructor
public class UrlService {
    private static final Pattern URL_TYPE_PATTERN = Pattern.compile("([a-zA-Z]*):?(.*)");

    private final ActivityManager activityManager;
    private final LocaleText localeText;

    /**
     * Process the given url.
     * This method will invoke the appropriate activity when processing the url.
     *
     * @param url The url to process.
     * @return Returns true if the url was processed with success and an activity has been invoked, else false.
     */
    public boolean process(String url) {
        Assert.notNull(url, "url cannot be null");
        var matcher = URL_TYPE_PATTERN.matcher(url);

        if (matcher.matches()) {
            var type = matcher.group(1);
            log.trace("Found type \"{}\" for url {}", type, url);

            if (isWebUrl(type)) {
                log.debug("Opening web url: {}", url);
                activityManager.register(new PlayVideoActivity() {
                    @Override
                    public String getUrl() {
                        return url;
                    }

                    @Override
                    public String getTitle() {
                        return "";
                    }

                    @Override
                    public boolean isSubtitlesEnabled() {
                        return false;
                    }
                });

                return true;
            } else if (isMagnetLink(type)) {
                log.debug("Opening magnet link: {}", url);
                activityManager.register((LoadUrlActivity) () -> url);

                return true;
            } else {
                var file = new File(url);

                // check if the url is a valid file
                if (file.exists()) {
                    try {
                        if (isVideoFile(file)) {
                            log.debug("Opening video file: {}", url);
                            activityManager.register(new PlayVideoActivity() {
                                @Override
                                public String getUrl() {
                                    return url;
                                }

                                @Override
                                public String getTitle() {
                                    return FilenameUtils.getBaseName(url);
                                }

                                @Override
                                public boolean isSubtitlesEnabled() {
                                    return false;
                                }
                            });

                            return true;
                        }
                    } catch (IOException ex) {
                        log.error("Failed to process url, " + ex.getMessage(), ex);
                        activityManager.register((ErrorNotificationActivity) () -> localeText.get(MediaMessage.VIDEO_FAILED_TO_OPEN));
                        return false;
                    }
                } else {
                    log.warn("Failed to process url, file \"{}\" does not exist", url);
                    activityManager.register((ErrorNotificationActivity) () -> localeText.get(MediaMessage.URL_FAILED_TO_PROCESS, url));
                    return false;
                }
            }
        }

        log.warn("Failed to process url, url \"{}\" is invalid", url);
        activityManager.register((ErrorNotificationActivity) () -> localeText.get(MediaMessage.URL_FAILED_TO_PROCESS, url));

        return false;
    }

    /**
     * Check if the given file is a video file.
     *
     * @param file The file to check.
     * @return Returns true if the given file is a video file, else false.
     * @throws IOException Is thrown when the file cannot be read.
     */
    public boolean isVideoFile(File file) throws IOException {
        Assert.notNull(file, "file cannot be null");
        var contentType = Files.probeContentType(file.toPath());

        if (contentType != null) {
            var format = contentType.split("/")[0];
            return format.equalsIgnoreCase("video");
        } else {
            return false;
        }
    }

    private boolean isWebUrl(String type) {
        Assert.notNull(type, "type cannot be null");
        return type.equalsIgnoreCase("http") || type.equalsIgnoreCase("https");
    }

    private boolean isMagnetLink(String type) {
        return type.equalsIgnoreCase("magnet");
    }
}
