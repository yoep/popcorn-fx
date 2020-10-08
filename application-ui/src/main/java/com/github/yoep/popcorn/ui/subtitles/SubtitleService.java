package com.github.yoep.popcorn.ui.subtitles;

import com.github.yoep.popcorn.ui.config.properties.PopcornProperties;
import com.github.yoep.popcorn.ui.config.properties.SubtitleProperties;
import com.github.yoep.popcorn.ui.media.providers.models.Episode;
import com.github.yoep.popcorn.ui.media.providers.models.Media;
import com.github.yoep.popcorn.ui.media.providers.models.Movie;
import com.github.yoep.popcorn.ui.media.providers.models.Show;
import com.github.yoep.popcorn.ui.settings.SettingsService;
import com.github.yoep.popcorn.ui.settings.models.ApplicationSettings;
import com.github.yoep.popcorn.ui.settings.models.SubtitleSettings;
import com.github.yoep.popcorn.ui.subtitles.models.*;
import de.timroes.axmlrpc.XMLRPCCallback;
import de.timroes.axmlrpc.XMLRPCClient;
import de.timroes.axmlrpc.XMLRPCException;
import de.timroes.axmlrpc.XMLRPCServerException;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.apache.commons.io.FileUtils;
import org.apache.commons.io.FilenameUtils;
import org.apache.commons.io.IOUtils;
import org.apache.commons.lang3.StringUtils;
import org.springframework.http.HttpMethod;
import org.springframework.scheduling.annotation.Async;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;
import org.springframework.util.StreamUtils;
import org.springframework.web.client.RestTemplate;

import javax.annotation.PostConstruct;
import javax.annotation.PreDestroy;
import java.io.File;
import java.io.FileOutputStream;
import java.io.IOException;
import java.nio.charset.Charset;
import java.nio.charset.IllegalCharsetNameException;
import java.nio.charset.UnsupportedCharsetException;
import java.text.MessageFormat;
import java.util.*;
import java.util.concurrent.CompletableFuture;
import java.util.function.Consumer;
import java.util.regex.Pattern;
import java.util.zip.ZipFile;

@Slf4j
@Service
@RequiredArgsConstructor
public class SubtitleService {
    private static final Pattern QUALITY_PATTERN = Pattern.compile("([0-9]{3,4})p");

    private final Map<String, List<SubtitleInfo>> cachedSubtitles = new HashMap<>();
    private final PopcornProperties popcornProperties;
    private final SettingsService settingsService;
    private final RestTemplate restTemplate;
    private final XMLRPCClient client;

    //region Methods

    /**
     * Download the SRT file for the given {@link SubtitleInfo}.
     *
     * @param subtitle The subtitle information file to download.
     * @return Returns the downloaded SRT file.
     */
    @Async
    public CompletableFuture<File> download(final SubtitleInfo subtitle, SubtitleMatcher matcher) {
        Assert.notNull(subtitle, "subtitle cannot be null");
        var subtitleFile = subtitle.getFile(matcher);

        return CompletableFuture.completedFuture(internalDownload(subtitleFile));
    }

    /**
     * Retrieve the subtitle for the given media.
     *
     * @param media The media to retrieve the subtitles of.
     * @return Returns the list of available subtitles for the media.
     */
    @Async
    public CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(final Movie media) {
        Assert.notNull(media, "media cannot be null");

        // check if the subtitles were already cached
        if (cachedSubtitles.containsKey(media.getId()))
            return CompletableFuture.completedFuture(cachedSubtitles.get(media.getId()));

        var completableFuture = new CompletableFuture<List<SubtitleInfo>>();
        var searchHandler = createSearchCallbackHandler(media, completableFuture);
        var loginHandler = createLoginCallbackHandler(token -> search(media, token, searchHandler), completableFuture);

        completableFuture.thenAccept(e -> cachedSubtitles.put(media.getId(), e));

        login(loginHandler);
        return completableFuture;
    }

    /**
     * Retrieve the subtitle for the given media.
     *
     * @param media   The media to retrieve the subtitles of.
     * @param episode The episode of the media to retrieve the subtitle of.
     * @return Returns the list of available subtitles for the media.
     */
    @Async
    public CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(final Show media, final Episode episode) {
        Assert.notNull(media, "media cannot be null");
        Assert.notNull(episode, "episode cannot be null");
        var cacheId = String.valueOf(episode.getId());

        // check if the subtitles were already cached
        if (cachedSubtitles.containsKey(cacheId))
            return CompletableFuture.completedFuture(cachedSubtitles.get(cacheId));

        var completableFuture = new CompletableFuture<List<SubtitleInfo>>();
        var searchHandler = createSearchCallbackHandler(media, completableFuture);
        var loginHandler = createLoginCallbackHandler(token -> search(media, episode, token, searchHandler), completableFuture);

        completableFuture.thenAccept(e -> cachedSubtitles.put(cacheId, e));

        login(loginHandler);
        return completableFuture;
    }

    /**
     * Retrieve the subtitles for the given filename.
     *
     * @param filename The filename to retrieve the subtitle for.
     * @return Returns the list of available subtitles for the given file.
     */
    @Async
    public CompletableFuture<List<SubtitleInfo>> retrieveSubtitles(final String filename) {
        Assert.hasText(filename, "filename cannot be empty");
        var completableFuture = new CompletableFuture<List<SubtitleInfo>>();
        var searchHandler = createSearchCallbackHandler(completableFuture);
        var loginHandler = createLoginCallbackHandler(token -> search(filename, token, searchHandler), completableFuture);

        login(loginHandler);
        return completableFuture;
    }

    /**
     * Parse the given SRT file to a list of {@link SubtitleIndex}'s.
     *
     * @param file     The SRT file to parse.
     * @param encoding The encoding of the SRT file.
     * @return Returns the parsed subtitle.
     */
    @Async
    public CompletableFuture<Subtitle> parse(File file, Charset encoding) {
        Assert.notNull(file, "file cannot be null");
        if (!file.exists())
            return CompletableFuture.failedFuture(
                    new SubtitleException(String.format("Failed to parse subtitle file, file \"%s\" does not exist", file.getAbsolutePath())));

        return CompletableFuture.completedFuture(internalParse(file, encoding));
    }

    /**
     * Download and parse the SRT file for the given {@link SubtitleInfo}.
     *
     * @param subtitleInfo The subtitle info to download and parse.
     * @return Returns the subtitle for the given subtitle info.
     */
    @Async
    public CompletableFuture<Subtitle> downloadAndParse(SubtitleInfo subtitleInfo, SubtitleMatcher matcher) {
        Assert.notNull(subtitleInfo, "subtitleInfo cannot be null");
        Assert.notNull(matcher, "matcher cannot be null");
        var subtitleFile = subtitleInfo.getFile(matcher);
        var file = subtitleInfo.isCustom() ? getCustomSubtitleFile(subtitleFile) : internalDownload(subtitleFile);
        var encoding = subtitleFile.getEncoding();

        return CompletableFuture.completedFuture(internalParse(subtitleInfo, file, encoding));
    }

    /**
     * Get the subtitle that needs to be selected by default for the given subtitles list.
     * This is based on the subtitle settings and tries to find the user's preferred language if it exist.
     * If the user's preferred language doesn't exist in the list, it returns the default {@link SubtitleInfo#none()} subtitle.
     *
     * @param subtitles The subtitle list to search for the preferred language.
     * @return Returns the subtitle that needs to be selected by default.
     */
    public SubtitleInfo getDefault(List<SubtitleInfo> subtitles) {
        Assert.notNull(subtitles, "subtitles cannot be null");

        // try to find the subtitle language from settings if it exists in the list
        // if not found, return the special none subtitle
        return getDefaultBase(subtitles)
                .orElseGet(SubtitleInfo::none);
    }

    /**
     * Get the subtitle that needs to be selected by default for the given subtitles list.
     * This is based on the subtitle settings and tries to find the user's preferred language if it exist or uses the interface language if not found.
     * If the user's preferred language doesn't exist in the list, it will use the interface language.
     * If both the user's preferred language and interface language don't exist, it returns the default {@link SubtitleInfo#none()} subtitle.
     *
     * @param subtitles The subtitle list to search for the preferred language.
     * @return Returns the subtitle that needs to be selected by default.
     */
    public SubtitleInfo getDefaultOrInterfaceLanguage(List<SubtitleInfo> subtitles) {
        Assert.notNull(subtitles, "subtitles cannot be null");
        var interfaceLanguage = getSettings().getUiSettings().getDefaultLanguage();

        // try to find the subtitle language from settings if it exists in the list
        // if not found, return the special none subtitle
        return getDefaultBase(subtitles)
                .orElseGet(() ->
                        subtitles.stream()
                                .filter(e -> e.getLanguage().getCode().equalsIgnoreCase(interfaceLanguage.getLanguage()))
                                .findFirst()
                                .orElseGet(SubtitleInfo::none));
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    private void init() {
        initializeListeners();
    }

    private void initializeListeners() {
        SubtitleSettings settings = getSubtitleSettings();

        settings.addListener(evt -> {
            if (SubtitleSettings.DIRECTORY_PROPERTY.equals(evt.getPropertyName())) {
                // clean old directory
                if (settings.isAutoCleaningEnabled())
                    cleanCacheDirectory((File) evt.getOldValue());
            }
        });
    }

    //endregion

    //region PreDestroy

    @PreDestroy
    private void destroy() {
        var settings = getSubtitleSettings();

        if (settings.isAutoCleaningEnabled() && settings.getDirectory().exists()) {
            cleanCacheDirectory(settings.getDirectory());
        }
    }

    //endregion

    //region Functions

    private XMLRPCCallback createLoginCallbackHandler(Consumer<String> onResponse, CompletableFuture<List<SubtitleInfo>> completableFuture) {
        return new XMLRPCCallback() {
            @Override
            public void onResponse(long id, Object result) {
                log.trace("Received login response from {}", popcornProperties.getSubtitle().getUrl());
                var response = (Map<String, Object>) result;
                var token = (String) response.get("token");
                var status = response.get("status");

                if (StringUtils.isNotEmpty(token)) {
                    onResponse.accept(token);
                } else {
                    var message = MessageFormat.format("Failed to retrieve subtitles, login failed (status: {0})", status);
                    completableFuture.completeExceptionally(new SubtitleException(message));
                }
            }

            @Override
            public void onError(long id, XMLRPCException error) {
                completableFuture.completeExceptionally(error);
            }

            @Override
            public void onServerError(long id, XMLRPCServerException error) {
                completableFuture.completeExceptionally(error);
            }
        };
    }

    private XMLRPCCallback createSearchCallbackHandler(final CompletableFuture<List<SubtitleInfo>> completableFuture) {
        return createSearchCallbackHandler(null, completableFuture);
    }

    private XMLRPCCallback createSearchCallbackHandler(final Media media, final CompletableFuture<List<SubtitleInfo>> completableFuture) {
        return new XMLRPCCallback() {
            @Override
            public void onResponse(long id, Object result) {
                List<SubtitleInfo> subtitles = new ArrayList<>();
                Map<String, Object> subData = (Map<String, Object>) result;

                // add the custom & default subtitles
                subtitles.add(SubtitleInfo.custom());
                subtitles.add(SubtitleInfo.none());

                if (subData != null && subData.get("data") != null && subData.get("data") instanceof Object[]) {
                    Object[] dataList = (Object[]) subData.get("data");
                    for (Object dataItem : dataList) {
                        Map<String, String> item = (Map<String, String>) dataItem;

                        // check if the subtitle file format is srt
                        // if not, skip this item
                        if (!item.get("SubFormat").equals("srt")) {
                            continue;
                        }

                        // check if the media year matches the returned item
                        if (media != null && !item.get("MovieYear").equals(media.getYear())) {
                            continue;
                        }

                        var mediaId = media != null ? media.getId() : "tt" + item.get("IDMovieImdb");
                        var url = item.get("SubDownloadLink").replace(".gz", ".srt");
                        var name = item.get("SubFileName");
                        var language = SubtitleLanguage.valueOfCode(item.get("ISO639").replace("pb", "pt-br"));
                        var encoding = parseSubEncoding(item.get("SubEncoding"));

                        // check if language is known within the subtitle languages
                        // if not known, ignore this subtitle and continue with the next one
                        if (language == null)
                            continue;

                        int downloads = Integer.parseInt(item.get("SubDownloadsCnt"));
                        int score = 0;

                        if (item.get("MatchedBy").equals("tag")) {
                            score += 50;
                        }
                        if (item.get("UserRank").equals("trusted")) {
                            score += 100;
                        }

                        var existingSubtitle = subtitles.stream()
                                .filter(e -> e.getLanguage() == language)
                                .findAny();
                        SubtitleInfo subtitle;

                        if (existingSubtitle.isPresent()) {
                            subtitle = existingSubtitle.get();
                        } else {
                            subtitle = new SubtitleInfo(mediaId, language);
                            subtitles.add(subtitle);
                        }

                        subtitle.addFile(SubtitleFile.builder()
                                .quality(parseSubtitleQuality(name).orElse(null))
                                .name(name)
                                .url(url)
                                .score(score)
                                .downloads(downloads)
                                .encoding(encoding)
                                .build());
                    }

                    // get the amount of found subtitles excluding the special ones
                    var totalFoundSubtitles = subtitles.stream()
                            .filter(e -> !e.isSpecial())
                            .count();

                    // always subtract the "none" subtitle from the count
                    if (media != null) {
                        log.debug("Found {} subtitles for \"{}\" media ({})", totalFoundSubtitles, media.getTitle(), media.getId());
                    } else {
                        log.debug("Found {} subtitles", totalFoundSubtitles);
                    }

                    completableFuture.complete(subtitles);
                } else {
                    String message;

                    if (media != null) {
                        message = MessageFormat.format("No subtitles found for \"{0}\" media ({1})", media.getTitle(), media.getId());
                    } else {
                        message = "No subtitles could be found";
                    }

                    completableFuture.completeExceptionally(new SubtitleException(message));
                }
            }

            @Override
            public void onError(long id, XMLRPCException error) {
                completableFuture.completeExceptionally(error);
            }

            @Override
            public void onServerError(long id, XMLRPCServerException error) {
                completableFuture.completeExceptionally(error);
            }
        };
    }

    /**
     * Login to server and get token
     *
     * @param callback XML RPC callback
     */
    private void login(XMLRPCCallback callback) {
        SubtitleProperties subtitleProperties = popcornProperties.getSubtitle();
        log.trace("Logging in to {}", subtitleProperties.getUrl());

        client.callAsync(callback, "LogIn", subtitleProperties.getUsername(), subtitleProperties.getPassword(), "en", subtitleProperties.getUserAgent());
    }

    private void search(Show show, Episode episode, String token, XMLRPCCallback callback) {
        Map<String, String> option = new HashMap<>();
        option.put("imdbid", show.getId().replace("tt", ""));
        option.put("season", String.valueOf(episode.getSeason()));
        option.put("episode", String.valueOf(episode.getEpisode()));
        option.put("sublanguageid", "all");

        client.callAsync(callback, "SearchSubtitles", token, new Object[]{option});
    }

    private void search(Movie movie, String token, XMLRPCCallback callback) {
        log.trace("Searching for \"{}\" movie subtitles ({})", movie.getTitle(), movie.getId());
        Map<String, String> option = new HashMap<>();
        option.put("imdbid", movie.getId().replace("tt", ""));
        option.put("sublanguageid", "all");

        client.callAsync(callback, "SearchSubtitles", token, new Object[]{option});
    }

    private void search(String filename, String token, XMLRPCCallback callback) {
        log.trace("Searching for \"{}\" filename subtitles", filename);
        Map<String, String> option = new HashMap<>();
        option.put("tag", filename);

        client.callAsync(callback, "SearchSubtitles", token, new Object[]{option});
    }

    private File getCustomSubtitleFile(SubtitleFile subtitleFile) {
        var url = subtitleFile.getUrl();

        // verify if the custom subtitle url is not empty
        if (StringUtils.isEmpty(url)) {
            throw new SubtitleException("Custom subtitle file url is empty");
        }

        var extension = FilenameUtils.getExtension(url);

        // check if the file is a zip file
        // if so, extract the zip and search for the .srt file
        if (extension.equalsIgnoreCase("zip")) {
            log.debug("Custom subtitle file is a zip, extracting and searching for subtitle file");
            try (var zipFile = new ZipFile(new File(url))) {
                var entries = zipFile.entries();
                var subtitleDirectory = getSubtitleSettings().getDirectory();

                // loop over each entry in the zip file
                // and search for the .srt file
                while (entries.hasMoreElements()) {
                    var entry = entries.nextElement();
                    var filename = entry.getName();

                    // check if the entry is a file
                    if (!entry.isDirectory()) {
                        var entryExtension = FilenameUtils.getExtension(filename);

                        if (entryExtension.equalsIgnoreCase("srt")) {
                            var destinationSubtitleFile = new File(subtitleDirectory + File.separator + filename);

                            // CVE-2018-1263, CVE-2018-16131
                            // verify that the zip file is not trying to leave the intended target directory
                            if (!isValidDestinationPath(destinationSubtitleFile, subtitleDirectory)) {
                                var message = MessageFormat.format("Unable to extract file \"{0}\", file is trying to leaving destination directory \"{1}\"",
                                        filename, subtitleDirectory.getAbsolutePath());
                                throw new SubtitleException(message);
                            }

                            // check if the destination file already exists
                            // if so, return the cached file
                            if (destinationSubtitleFile.exists()) {
                                log.debug("Using cached extracted archive subtitle file {}", destinationSubtitleFile.getAbsolutePath());
                                return destinationSubtitleFile;
                            }

                            try (var inputStream = zipFile.getInputStream(entry)) {
                                var outputStream = new FileOutputStream(destinationSubtitleFile);

                                log.trace("Copying subtitle file from archive to {}", destinationSubtitleFile.getAbsolutePath());
                                IOUtils.copy(inputStream, outputStream);
                            }

                            log.debug("Subtitle file has successfully been extracted from the archive to {}", destinationSubtitleFile.getAbsolutePath());
                            return destinationSubtitleFile;
                        }
                    }
                }

                // the subtitle couldn't be found
                // so we raise an error so that the subtitle track gets updated to disabled
                var message = MessageFormat.format("No srt file could be found in \"{0}\" the custom subtitle file", url);
                throw new SubtitleException(message);
            } catch (IOException ex) {
                throw new SubtitleException("Failed to extract subtitle file from archive, " + ex.getMessage(), ex);
            }
        }

        return new File(url);
    }

    private boolean isValidDestinationPath(File destinationFile, File destinationDirectory) throws IOException {
        var destinationDirectoryPath = destinationDirectory.getCanonicalPath();
        var destinationFilePath = destinationFile.getCanonicalPath();

        return destinationFilePath.startsWith(destinationDirectoryPath);
    }

    private File internalDownload(SubtitleFile subtitle) {
        File storageFile = getStorageFile(subtitle);
        File subtitleFile;

        // check if the subtitle file was already downloaded before, if so, return the cached file
        if (storageFile.exists()) {
            log.debug("Returning cached subtitle file \"{}\"", storageFile.getAbsolutePath());
            subtitleFile = storageFile;
        } else {
            log.debug("Downloading subtitle file \"{}\" to \"{}\"", subtitle.getUrl(), storageFile.getAbsolutePath());
            subtitleFile = restTemplate.execute(subtitle.getUrl(), HttpMethod.GET, null, response -> {
                StreamUtils.copy(response.getBody(), new FileOutputStream(storageFile));
                return storageFile;
            });
        }

        return subtitleFile;
    }

    private Subtitle internalParse(File file, Charset encoding) {
        return internalParse(null, file, encoding);
    }

    private Subtitle internalParse(SubtitleInfo subtitleInfo, File file, Charset encoding) {
        var indexes = SrtParser.parse(file, encoding);

        return new Subtitle(subtitleInfo, file, indexes);
    }

    private Charset parseSubEncoding(String encoding) {
        // check if the charset encoding is not empty
        // if it is empty, return the default charset instead
        if (StringUtils.isEmpty(encoding))
            return Charset.defaultCharset();

        try {
            return Charset.forName(encoding);
        } catch (IllegalCharsetNameException | UnsupportedCharsetException ex) {
            log.warn("Failed to parse subtitle encoding, " + ex.getMessage(), ex);
            return Charset.defaultCharset();
        }
    }

    private Optional<Integer> parseSubtitleQuality(String name) {
        var matcher = QUALITY_PATTERN.matcher(name);

        if (matcher.find()) {
            var quality = matcher.group(1);

            return Optional.of(Integer.parseInt(quality));
        }

        return Optional.empty();
    }

    private Optional<SubtitleInfo> getDefaultBase(List<SubtitleInfo> subtitles) {
        var settings = getSubtitleSettings();

        return subtitles.stream()
                .filter(e -> e.getLanguage() == settings.getDefaultSubtitle())
                .findFirst();
    }

    private SubtitleSettings getSubtitleSettings() {
        return getSettings().getSubtitleSettings();
    }

    private ApplicationSettings getSettings() {
        return settingsService.getSettings();
    }

    private File getStorageFile(SubtitleFile subtitle) {
        var filename = FilenameUtils.getName(subtitle.getUrl());
        var subtitleDirectory = getSubtitleSettings().getDirectory();

        // make sure the subtitle directory exists
        subtitleDirectory.mkdirs();

        return new File(subtitleDirectory.getAbsolutePath() + File.separator + filename);
    }

    private void cleanCacheDirectory(File directory) {
        try {
            log.info("Cleaning subtitles directory {}", directory);
            FileUtils.cleanDirectory(directory);
        } catch (IOException ex) {
            log.error(ex.getMessage(), ex);
        }
    }

    //endregion
}
