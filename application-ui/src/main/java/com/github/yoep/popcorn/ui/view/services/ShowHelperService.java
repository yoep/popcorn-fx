package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.media.Episode;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import com.github.yoep.popcorn.backend.media.filters.model.Season;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.time.format.DateTimeFormatter;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.ExecutionException;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;
import java.util.stream.Collectors;

@Slf4j
@RequiredArgsConstructor
public class ShowHelperService {
    public static final DateTimeFormatter AIRED_DATE_PATTERN = DateTimeFormatter.ofPattern("EEEE, MMMM dd, yyyy hh:mm a");

    private final LocaleText localeText;
    private final WatchedService watchedService;

    //region Methods

    /**
     * Get the season of the given media.
     *
     * @param media The media to retrieve the seasons of.
     * @return Returns the list of season for the media.
     */
    public List<Season> getSeasons(ShowDetails media) {
        Objects.requireNonNull(media, "media cannot be null");
        var seasons = new ArrayList<Season>();

//        for (int i = 1; i <= media.getNumberOfSeasons(); i++) {
//            seasons.add(new Season(i, localeText.get(DetailsMessage.SEASON, i)));
//        }

        return seasons;
    }

    /**
     * Get the media episodes for the given season.
     *
     * @param season The season to show the episodes of.
     * @param media  The media that contains the episodes.
     * @return Returns the list of episodes for the season.
     */
    public List<Episode> getSeasonEpisodes(Season season, ShowDetails media) {
        Objects.requireNonNull(season, "season cannot be null");
        Objects.requireNonNull(media, "media cannot be null");

        return media.getEpisodes().stream()
                .filter(Objects::nonNull)
//                .filter(e -> e.getSeason() == season.getSeason())
//                .sorted(Comparator.comparing(Episode::getEpisode))
                .collect(Collectors.toList());
    }

    /**
     * Get the first unwatched season for the given media.
     *
     * @param seasons The season to select from.
     * @param media   The media of the seasons.
     * @return Returns the unwatched season or last season if all seasons have been watched.
     */
    public Season getUnwatchedSeason(List<Season> seasons, ShowDetails media) {
        Objects.requireNonNull(seasons, "seasons cannot be null");
        Objects.requireNonNull(media, "media cannot be null");

        for (var season : seasons.stream().sorted().toList()) {
            try {
                if (isSeasonWatched(season, media).get(10, TimeUnit.SECONDS)) {
                    return season;
                }
            } catch (ExecutionException | TimeoutException | InterruptedException ex) {
                log.error("Failed to retrieve is watched", ex);
            }
        }

        return seasons.get(seasons.size() - 1);
    }

    /**
     * Get the first unwatched episode from the episodes list.
     *
     * @param episodes The episodes list to select from.
     * @param season
     * @return Returns the first unwatched episode, or the last episode if all episodes have been watched.
     */
    public CompletableFuture<Episode> getUnwatchedEpisode(List<Episode> episodes, Season season) {
        Objects.requireNonNull(episodes, "episodes cannot be null");
        var future = new CompletableFuture<Episode>();

        CompletableFuture.supplyAsync(() -> {
            for (var episode : episodes.stream()
                    .sorted()
                    .filter(Objects::nonNull)
                    .filter(e -> e.season() == season.season())
                    .toList()) {
                try {
                    if (watchedService.isWatched(episode).get(10, TimeUnit.SECONDS)) {
                        future.complete(episode);
                        return null;
                    }
                } catch (ExecutionException | TimeoutException | InterruptedException ex) {
                    log.error("Failed to retrieve is watched", ex);
                }
            }

            future.complete(episodes.get(0));
            return null;
        });

        return future;
    }

    //endregion

    //region Functions

    private CompletableFuture<Boolean> isSeasonWatched(Season season, ShowDetails media) {
        CompletableFuture<Boolean>[] futures = getSeasonEpisodes(season, media).stream()
                .map(watchedService::isWatched)
                .toArray(CompletableFuture[]::new);
        return CompletableFuture.allOf(futures).thenApply(v -> Arrays.stream(futures)
                .allMatch(CompletableFuture::join));
    }

    //endregion
}
