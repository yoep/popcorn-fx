package com.github.yoep.popcorn.services;

import com.github.yoep.popcorn.config.properties.PopcornProperties;
import com.github.yoep.popcorn.config.properties.SubtitleProperties;
import com.github.yoep.popcorn.media.providers.models.Episode;
import com.github.yoep.popcorn.media.providers.models.Media;
import com.github.yoep.popcorn.media.providers.models.Movie;
import com.github.yoep.popcorn.media.providers.models.Show;
import com.github.yoep.popcorn.models.Subtitle;
import de.timroes.axmlrpc.XMLRPCCallback;
import de.timroes.axmlrpc.XMLRPCClient;
import de.timroes.axmlrpc.XMLRPCException;
import de.timroes.axmlrpc.XMLRPCServerException;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.http.HttpMethod;
import org.springframework.scheduling.annotation.Async;
import org.springframework.stereotype.Service;
import org.springframework.util.StreamUtils;
import org.springframework.web.client.RestTemplate;

import java.io.File;
import java.io.FileOutputStream;
import java.util.*;
import java.util.concurrent.CompletableFuture;

@Slf4j
@Service
@RequiredArgsConstructor
public class SubtitleService {
    private final List<Long> ongoingCalls = new ArrayList<>();
    private final PopcornProperties popcornProperties;
    private final RestTemplate restTemplate;
    private final XMLRPCClient client;

    @Async
    public void download(final Media media, final String languageCode) {
        media.getSubtitles().stream()
                .filter(e -> e.getLanguage().equalsIgnoreCase(languageCode))
                .findFirst()
                .ifPresent(subtitle -> {
                    restTemplate.execute(subtitle.getUrl(), HttpMethod.GET, null, response -> {
                        File ret = File.createTempFile("download", "tmp");
                        StreamUtils.copy(response.getBody(), new FileOutputStream(ret));
                        return ret;
                    });
                });
    }

    @Async
    public CompletableFuture<List<Subtitle>> getList(final Movie movie) {
        CompletableFuture<List<Subtitle>> completableFuture = new CompletableFuture<>();

        login(new XMLRPCCallback() {
            @Override
            public void onResponse(long id, Object result) {
                log.trace("Received login response from {}", popcornProperties.getSubtitle().getUrl());
                Map<String, Object> response = (Map<String, Object>) result;
                String token = (String) response.get("token");

                if (token != null && !token.isEmpty()) {
                    search(movie, token, new XMLRPCCallback() {
                        @Override
                        public void onResponse(long id, Object result) {
                            List<Subtitle> subtitles = new ArrayList<>();
                            Map<String, Object> subData = (Map<String, Object>) result;

                            if (subData != null && subData.get("data") != null && subData.get("data") instanceof Object[]) {
                                Object[] dataList = (Object[]) subData.get("data");
                                for (Object dataItem : dataList) {
                                    Map<String, String> item = (Map<String, String>) dataItem;
                                    if (!item.get("SubFormat").equals("srt")) {
                                        continue;
                                    }

                                    // imdb & year check
                                    if (Integer.parseInt(item.get("IDMovieImdb")) != Integer.parseInt(movie.getImdbId().replace("tt", ""))) {
                                        continue;
                                    }
                                    if (!item.get("MovieYear").equals(movie.getYear())) {
                                        continue;
                                    }

                                    String url = item.get("SubDownloadLink").replace(".gz", ".srt");
                                    String lang = item.get("ISO639").replace("pb", "pt-br");
                                    int downloads = Integer.parseInt(item.get("SubDownloadsCnt"));
                                    int score = 0;

                                    if (item.get("MatchedBy").equals("tag")) {
                                        score += 50;
                                    }
                                    if (item.get("UserRank").equals("trusted")) {
                                        score += 100;
                                    }

                                    Optional<Subtitle> subtitle = subtitles.stream()
                                            .filter(e -> e.getLanguage().equalsIgnoreCase(lang))
                                            .findAny();

                                    if (subtitle.isPresent()) {
                                        Subtitle sub = subtitle.get();

                                        if (score > sub.getScore() || (score == sub.getScore() && downloads > sub.getDownloads())) {
                                            sub.setUrl(url);
                                            sub.setScore(score);
                                            sub.setDownloads(downloads);
                                        }
                                    } else {
                                        subtitles.add(new Subtitle(lang, url, score, downloads));
                                    }
                                }

                                log.debug("Found {} subtitles for \"{}\" movie ({})", subtitles.size(), movie.getTitle(), movie.getImdbId());
                                completableFuture.complete(subtitles);
                            } else {
                                completableFuture.completeExceptionally(new XMLRPCException("No subs found"));
                                removeCall(id);
                            }
                        }

                        @Override
                        public void onError(long id, XMLRPCException error) {
                            completableFuture.completeExceptionally(error);
                            removeCall(id);
                        }

                        @Override
                        public void onServerError(long id, XMLRPCServerException error) {
                            completableFuture.completeExceptionally(error);
                            removeCall(id);
                        }
                    });
                } else {
                    completableFuture.completeExceptionally(new XMLRPCException("Token not correct"));
                }
            }

            @Override
            public void onError(long id, XMLRPCException error) {
                log.error(error.getMessage(), error);
                completableFuture.completeExceptionally(error);
                removeCall(id);
            }

            @Override
            public void onServerError(long id, XMLRPCServerException error) {
                log.error(error.getMessage(), error);
                completableFuture.completeExceptionally(error);
                removeCall(id);
            }
        });

        return completableFuture;
    }

    private void removeCall(long callId) {
        synchronized (ongoingCalls) {
            ongoingCalls.remove(callId);
        }
    }

    /**
     * Login to server and get token
     *
     * @param callback XML RPC callback
     */
    private void login(XMLRPCCallback callback) {
        SubtitleProperties subtitleProperties = popcornProperties.getSubtitle();
        log.trace("Logging in to {}", subtitleProperties.getUrl());

        long callId = client.callAsync(callback, "LogIn", "", "", "en", subtitleProperties.getUserAgent());
        synchronized (ongoingCalls) {
            ongoingCalls.add(callId);
        }
    }

    /**
     * @param episode  Episode
     * @param token    Login token
     * @param callback XML RPC callback callback
     */
    private void search(Show show, Episode episode, String token, XMLRPCCallback callback) {
        Map<String, String> option = new HashMap<>();
        option.put("imdbid", show.getImdbId().replace("tt", ""));
        option.put("season", String.format(Locale.US, "%d", episode.getSeason()));
        option.put("episode", String.format(Locale.US, "%d", episode.getEpisode()));
        option.put("sublanguageid", "all");
        long callId = client.callAsync(callback, "SearchSubtitles", token, new Object[]{option});
        synchronized (ongoingCalls) {
            ongoingCalls.add(callId);
        }
    }

    /**
     * @param movie    Movie
     * @param token    Login token
     * @param callback XML RPC callback callback
     */
    private void search(Movie movie, String token, XMLRPCCallback callback) {
        log.trace("Searching for \"{}\" movie subtitles ({})", movie.getTitle(), movie.getImdbId());
        Map<String, String> option = new HashMap<>();
        option.put("imdbid", movie.getImdbId().replace("tt", ""));
        option.put("sublanguageid", "all");
        long callId = client.callAsync(callback, "SearchSubtitles", token, new Object[]{option});
        synchronized (ongoingCalls) {
            ongoingCalls.add(callId);
        }
    }
}
