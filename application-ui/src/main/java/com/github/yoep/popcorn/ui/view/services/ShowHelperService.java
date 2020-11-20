package com.github.yoep.popcorn.ui.view.services;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.ui.media.providers.models.Episode;
import com.github.yoep.popcorn.ui.media.providers.models.Show;
import com.github.yoep.popcorn.ui.media.watched.WatchedService;
import com.github.yoep.popcorn.ui.messages.DetailsMessage;
import com.github.yoep.popcorn.ui.view.models.Season;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;
import org.springframework.util.Assert;
import org.springframework.util.CollectionUtils;

import java.time.format.DateTimeFormatter;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;
import java.util.Objects;
import java.util.stream.Collectors;

@Slf4j
@Service
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
    public List<Season> getSeasons(Show media) {
        Assert.notNull(media, "media cannot be null");
        var seasons = new ArrayList<Season>();

        for (int i = 1; i <= media.getNumberOfSeasons(); i++) {
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
    public List<Episode> getSeasonEpisodes(Season season, Show media) {
        Assert.notNull(season, "season cannot be null");
        Assert.notNull(media, "media cannot be null");

        return media.getEpisodes().stream()
                .filter(Objects::nonNull)
                .filter(e -> e.getSeason() == season.getSeason())
                .sorted(Comparator.comparing(Episode::getEpisode))
                .collect(Collectors.toList());
    }

    /**
     * Get the first unwatched season for the given media.
     *
     * @param seasons The season to select from.
     * @param media   The media of the seasons.
     * @return Returns the unwatched season or last season if all seasons have been watched.
     */
    public Season getUnwatchedSeason(List<Season> seasons, Show media) {
        Assert.notNull(seasons, "seasons cannot be null");
        Assert.notNull(media, "media cannot be null");

        return seasons.stream()
                .filter(e -> !isSeasonWatched(e, media))
                .findFirst()
                .orElseGet(() -> CollectionUtils.lastElement(seasons));
    }

    /**
     * Get the first unwatched episode from the episodes list.
     *
     * @param episodes The episodes list to select from.
     * @return Returns the first unwatched episode, or the last episode if all episodes have been watched.
     */
    public Episode getUnwatchedEpisode(List<Episode> episodes) {
        Assert.notNull(episodes, "episodes cannot be null");

        return episodes.stream()
                .filter(Objects::nonNull)
                .filter(e -> !watchedService.isWatched(e))
                .findFirst()
                .orElseGet(() -> CollectionUtils.lastElement(episodes));
    }

    //endregion

    //region Functions

    private boolean isSeasonWatched(Season season, Show media) {
        return getSeasonEpisodes(season, media).stream()
                .allMatch(watchedService::isWatched);
    }

    //endregion
}
