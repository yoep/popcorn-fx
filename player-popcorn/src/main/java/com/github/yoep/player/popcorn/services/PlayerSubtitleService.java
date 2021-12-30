package com.github.yoep.player.popcorn.services;

import com.github.yoep.player.popcorn.listeners.PlayerSubtitleListener;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.models.SubtitleInfo;
import lombok.RequiredArgsConstructor;
import lombok.extern.slf4j.Slf4j;
import org.springframework.stereotype.Service;

import javax.annotation.PostConstruct;

@Slf4j
@Service
@RequiredArgsConstructor
public class PlayerSubtitleService extends AbstractListenerService<PlayerSubtitleListener> {
    private final SubtitleService subtitleService;
    private final SubtitleManagerService subtitleManagerService;

    //region Methods

    public void updateSubtitleSizeWithSizeOffset(int pixelChange) {
        subtitleManagerService.setSubtitleOffset(pixelChange);
    }

    public void updateActiveSubtitle(SubtitleInfo subtitleInfo) {
        subtitleManagerService.setSubtitle(subtitleInfo);
    }

    //endregion

    //region PostConstruct

    @PostConstruct
    void init() {
        subtitleService.activeSubtitleProperty().addListener((observableValue, subtitle, newSubtitle) ->
                invokeListeners(e -> e.onActiveSubtitleChanged(newSubtitle.getSubtitleInfo().orElse(SubtitleInfo.none()))));
    }

    //endregion

    //region Functions


    //endregion
}
