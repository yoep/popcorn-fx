package com.github.yoep.popcorn.ui.view.controllers.common.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.ui.view.controls.BackgroundImageCover;
import com.github.yoep.popcorn.ui.view.controls.Stars;
import com.github.yoep.popcorn.ui.view.listeners.DetailsComponentListener;
import com.github.yoep.popcorn.ui.view.services.DetailsComponentService;
import com.github.yoep.popcorn.ui.view.services.ImageService;
import javafx.scene.control.Label;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;

import java.util.concurrent.atomic.AtomicReference;

import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.doAnswer;
import static org.mockito.Mockito.lenient;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class MovieDetailsComponentTest {
    @Mock
    private DetailsComponentService service;
    @Mock
    private LocaleText localeText;
    @Mock
    private ImageService imageService;
    @Mock
    private PlayerManagerService playerService;
    @Mock
    private SubtitleService subtitleService;
    @Mock
    private FxLib fxLib;
    @Mock
    private SubtitleInfo.ByReference subtitleNone;
    @InjectMocks
    private MovieDetailsComponent component;

    private final AtomicReference<DetailsComponentListener> listenerHolder = new AtomicReference<>();

    @BeforeEach
    void setUp() {
        doAnswer(invocation -> {
            listenerHolder.set(invocation.getArgument(0, DetailsComponentListener.class));
            return null;
        }).when(service).addListener(isA(DetailsComponentListener.class));
        lenient().when(fxLib.subtitle_none()).thenReturn(subtitleNone);
        lenient().when(subtitleNone.getLanguage()).thenReturn(SubtitleLanguage.NONE);

        component.title = new Label("title");
        component.overview = new Label("overview");
        component.year = new Label("year");
        component.duration = new Label("duration");
        component.genres = new Label("genres");
        component.backgroundImage = new BackgroundImageCover();
        component.ratingStars = new Stars();
    }


}