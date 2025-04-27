package com.github.yoep.popcorn.ui.view.services;

import com.github.yoep.popcorn.backend.media.Episode;
import com.github.yoep.popcorn.backend.media.ShowDetails;
import com.github.yoep.popcorn.backend.media.filters.model.Season;
import com.github.yoep.popcorn.backend.media.watched.WatchedService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;

import java.time.format.DateTimeFormatter;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
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

        for (int i = 1; i <= media.proto().getNumberOfSeasons(); i++) {
            seasons.add(new Season(i, localeText.get(DetailsMessage.SEASON, i)));
        }

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
                .filter(e -> e.season() == season.season())
                .sorted(Comparator.comparing(Episode::episode))
                .collect(Collectors.toList());
    }

    /**
     * Get the first unwatched season for the given media.
     *
     * @param seasons The season to select from.
     * @param media   The media of the seasons.
     * @return Returns the unwatched season or last season if all seasons have been watched.
     */
    public CompletableFuture<Season> getUnwatchedSeason(List<Season> seasons, ShowDetails media) {
        Objects.requireNonNull(seasons, "seasons cannot be null");
        Objects.requireNonNull(media, "media cannot be null");
        var sortedSeasons = seasons.stream().sorted().toList();
        List<CompletableFuture<EnhancedSeason>> seasonFutures = sortedSeasons.stream()
                .map(season -> isSeasonWatched(season, media))
                .toList();

        return CompletableFuture.allOf(seasonFutures.toArray(new CompletableFuture[0]))
                .thenApply(v -> seasonFutures.stream().map(CompletableFuture::join)
                        .filter(e -> !e.isWatched)
                        .findFirst()
                        .map(EnhancedSeason::season)
                        .orElse(sortedSeasons.getLast()));
    }

    /**
     * Get the first unwatched episode from the episodes list.
     *
     * @param episodes The episodes list to select from.
     * @param season   The season to retrieve the first unwatched episode of.
     * @return Returns the first unwatched episode, or the last episode if all episodes have been watched.
     */
    public CompletableFuture<Episode> getUnwatchedEpisode(List<Episode> episodes, Season season) {
        Objects.requireNonNull(episodes, "episodes cannot be null");
        var filteredEpisodes = episodes.stream()
                .sorted()
                .filter(Objects::nonNull)
                .filter(e -> e.season() == season.season())
                .toList();

        List<CompletableFuture<EnhancedEpisode>> futures = filteredEpisodes.stream()
                .map(episode -> watchedService.isWatched(episode)
                        .thenApply(isWatched -> new EnhancedEpisode(episode, isWatched)))
                .toList();

        return CompletableFuture.allOf(futures.toArray(new CompletableFuture[0]))
                .thenApply(v -> futures.stream()
                        .map(CompletableFuture::join)
                        .filter(e -> !e.isWatched())
                        .findFirst()
                        .map(EnhancedEpisode::episode)
                        .orElse(filteredEpisodes.getFirst()))
                .exceptionally(ex -> {
                    log.warn("Failed to retrieve watched state of episodes", ex);
                    return filteredEpisodes.getFirst();
                });
    }

    //endregion

    //region Functions

    private CompletableFuture<EnhancedSeason> isSeasonWatched(Season season, ShowDetails media) {
        List<CompletableFuture<Boolean>> episodeWatchStatuses = getSeasonEpisodes(season, media).stream()
                .map(watchedService::isWatched)
                .toList();

        return CompletableFuture
                .allOf(episodeWatchStatuses.toArray(new CompletableFuture[0]))
                .thenApply(v -> {
                    var isWatched = episodeWatchStatuses.stream()
                            .allMatch(CompletableFuture::join);

                    return new EnhancedSeason(season, isWatched);
                });
    }

    private record EnhancedEpisode(Episode episode, boolean isWatched) {
    }

    private record EnhancedSeason(Season season, boolean isWatched) {
    }

    //endregion
}
