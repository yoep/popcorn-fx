package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.events.ErrorNotificationEvent;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.InfoNotificationEvent;
import com.github.yoep.popcorn.backend.loader.LoaderService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.OpenMagnetLinkEvent;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.messages.MediaMessage;
import javafx.application.Application;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FilenameUtils;
import org.apache.commons.lang3.StringUtils;

import java.io.File;
import java.io.IOException;
import java.nio.file.Files;
import java.util.Objects;
import java.util.regex.Pattern;

@Slf4j
public class UrlService {
    private static final Pattern URL_TYPE_PATTERN = Pattern.compile("([a-zA-Z]*):?(.*)");

    private final EventPublisher eventPublisher;
    private final Application application;
    private final LocaleText localeText;
    private final LoaderService loaderService;

    public UrlService(EventPublisher eventPublisher, Application application, LocaleText localeText, LoaderService loaderService) {
        this.eventPublisher = eventPublisher;
        this.application = application;
        this.localeText = localeText;
        this.loaderService = loaderService;
        init();
    }

    //region Methods

    /**
     * Open the given url link.
     *
     * @param url The url link to open.
     */
    public void open(String url) {
        Objects.requireNonNull(url, "url cannot be null");

        try {
            application.getHostServices().showDocument(url);
            eventPublisher.publishEvent(new InfoNotificationEvent(this, localeText.get(DetailsMessage.MAGNET_LINK_OPENING)));
        } catch (Exception ex) {
            log.error(ex.getMessage(), ex);
            eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(DetailsMessage.MAGNET_LINK_FAILED_TO_OPEN)));
        }
    }

    /**
     * Process the given url.
     * This method will invoke the appropriate activity when processing the url.
     *
     * @param url The url to process.
     * @return Returns true if the url was processed with success and an activity has been invoked, else false.
     */
    public boolean process(String url) {
        // check if the url is empty
        // if so, ignore this process action
        if (StringUtils.isBlank(url))
            return false;

        var matcher = URL_TYPE_PATTERN.matcher(url);

        if (matcher.matches()) {
            var type = matcher.group(1);
            log.trace("Found type \"{}\" for url {}", type, url);

            if (isWebUrl(type)) {
                log.debug("Opening web url: {}", url);
                loaderService.load(url);
                return true;
            } else if (isMagnetLink(type)) {
                log.debug("Opening magnet link: {}", url);
                loaderService.load(url);

                return true;
            } else {
                var file = new File(url);

                // check if the url is a valid file
                if (file.exists()) {
                    try {
                        if (isVideoFile(file)) {
                            log.debug("Opening video file: {}", url);
                            loaderService.load(url);
                            return true;
                        }
                    } catch (IOException ex) {
                        log.error("Failed to process url, " + ex.getMessage(), ex);
                        eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(MediaMessage.VIDEO_FAILED_TO_OPEN)));
                        return false;
                    }
                } else {
                    log.warn("Failed to process url, file \"{}\" does not exist", url);
                    eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(MediaMessage.URL_FAILED_TO_PROCESS, url)));
                    return false;
                }
            }
        }

        log.warn("Failed to process url, url \"{}\" is invalid", url);
        eventPublisher.publishEvent(new ErrorNotificationEvent(this, localeText.get(MediaMessage.URL_FAILED_TO_PROCESS, url)));

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
        Objects.requireNonNull(file, "file cannot be null");
        var contentType = Files.probeContentType(file.toPath());

        if (contentType != null) {
            var format = contentType.split("/")[0];
            return format.equalsIgnoreCase("video");
        } else {
            var extension = FilenameUtils.getExtension(file.getName());
            return "mkv".equalsIgnoreCase(extension);
        }
    }

    //endregion

    //region Functions

    private void init() {
        eventPublisher.register(OpenMagnetLinkEvent.class, event -> {
            open(event.getUrl());
            return event;
        });
    }

    private boolean isWebUrl(String type) {
        Objects.requireNonNull(type, "type cannot be null");
        return type.equalsIgnoreCase("http") || type.equalsIgnoreCase("https");
    }

    private boolean isMagnetLink(String type) {
        return type.equalsIgnoreCase("magnet");
    }

    //endregion
}
