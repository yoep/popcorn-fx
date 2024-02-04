package com.github.yoep.popcorn.ui.view.controllers.desktop.components;

import com.github.spring.boot.javafx.text.LocaleText;
import com.github.yoep.popcorn.backend.FxLib;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentFileInfo;
import com.github.yoep.popcorn.backend.adapters.torrent.model.TorrentInfo;
import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.events.ShowTorrentDetailsEvent;
import com.github.yoep.popcorn.backend.loader.LoaderService;
import com.github.yoep.popcorn.backend.settings.models.subtitles.SubtitleLanguage;
import com.github.yoep.popcorn.backend.subtitles.SubtitlePickerService;
import com.github.yoep.popcorn.backend.subtitles.SubtitleService;
import com.github.yoep.popcorn.backend.subtitles.model.SubtitleInfo;
import com.github.yoep.popcorn.ui.torrent.TorrentCollectionService;
import com.github.yoep.popcorn.ui.view.controls.PlayerDropDownButton;
import com.github.yoep.popcorn.ui.view.controls.SubtitleDropDownButton;
import javafx.scene.control.Button;
import javafx.scene.control.ListView;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.springframework.core.io.ByteArrayResource;
import org.springframework.core.io.ClassPathResource;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.Collections;
import java.util.ResourceBundle;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.TimeoutException;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class DetailsTorrentComponentTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private TorrentCollectionService torrentCollectionService;
    @Mock
    private LocaleText localeText;
    @Mock
    private PlayerManagerService playerManagerService;
    @Mock
    private SubtitlePickerService subtitlePickerService;
    @Mock
    private SubtitleService subtitleService;
    @Mock
    private LoaderService loaderService;
    @Mock
    private FxLib fxLib;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private DetailsTorrentComponent component;

    @BeforeEach
    void setUp() {
        component.fileShadow = new Pane();
        component.torrentList = new ListView<>();
        component.subtitleButton = new SubtitleDropDownButton();
        component.playerButton = new PlayerDropDownButton();
        component.storeTorrentButton = new Button();
    }

    @Test
    void testInitialize() {
        var subtitleNone = mock(SubtitleInfo.class);
        var subtitleCustom = mock(SubtitleInfo.class);
        when(subtitleNone.getLanguage()).thenReturn(SubtitleLanguage.NONE);
        when(subtitleNone.getFlagResource()).thenReturn(new ByteArrayResource(new byte[0]));
        when(subtitleCustom.getLanguage()).thenReturn(SubtitleLanguage.CUSTOM);
        when(subtitleCustom.getFlagResource()).thenReturn(new ByteArrayResource(new byte[0]));
        when(subtitleService.none()).thenReturn(subtitleNone);
        when(subtitleService.custom()).thenReturn(subtitleCustom);

        component.initialize(url, resourceBundle);
        WaitForAsyncUtils.waitForFxEvents();

        verify(subtitleService, atLeastOnce()).none();
        assertEquals(subtitleNone, component.subtitleButton.getSelectedItem().get());
    }

    @Test
    void testOnShowTorrentDetailsEvent() throws TimeoutException {
        var filename = " lorem ipsum dolor.mp4";
        var torrent = mock(TorrentInfo.class);
        var fileInfo = mock(TorrentFileInfo.class);
        var subtitleNone = mock(SubtitleInfo.class);
        var subtitleCustom = mock(SubtitleInfo.class);
        when(subtitleNone.getLanguage()).thenReturn(SubtitleLanguage.NONE);
        when(subtitleNone.getFlagResource()).thenReturn(new ByteArrayResource(new byte[0]));
        when(subtitleCustom.getLanguage()).thenReturn(SubtitleLanguage.CUSTOM);
        when(subtitleCustom.getFlagResource()).thenReturn(new ByteArrayResource(new byte[0]));
        when(subtitleService.none()).thenReturn(subtitleNone);
        when(subtitleService.custom()).thenReturn(subtitleCustom);
        when(torrent.getFiles()).thenReturn(Collections.singletonList(fileInfo));
        when(fileInfo.getFilename()).thenReturn(filename);
        component.initialize(url, resourceBundle);

        eventPublisher.publish(new ShowTorrentDetailsEvent(this, "", torrent));
        WaitForAsyncUtils.waitForFxEvents();

        WaitForAsyncUtils.waitFor(200, TimeUnit.MILLISECONDS, () -> component.torrentList.getItems().contains(fileInfo));
    }

    @Test
    void testOnFileInfoClicked() {
        var torrent = mock(TorrentInfo.class);
        var fileInfo = mock(TorrentFileInfo.class);
        var subtitleNone = mock(SubtitleInfo.class);
        var subtitleCustom = mock(SubtitleInfo.class);
        when(subtitleNone.getLanguage()).thenReturn(SubtitleLanguage.NONE);
        when(subtitleNone.getFlagResource()).thenReturn(new ByteArrayResource(new byte[0]));
        when(subtitleCustom.getLanguage()).thenReturn(SubtitleLanguage.CUSTOM);
        when(subtitleCustom.getFlagResource()).thenReturn(new ByteArrayResource(new byte[0]));
        when(subtitleService.none()).thenReturn(subtitleNone);
        when(subtitleService.custom()).thenReturn(subtitleCustom);
        component.initialize(url, resourceBundle);

        eventPublisher.publish(new ShowTorrentDetailsEvent(this, "", torrent));
        component.onFileInfoClicked(fileInfo);

        verify(loaderService).load(torrent, fileInfo);
    }

    private SubtitleInfo createSubtitle() {
        var subtitleInfo = mock(SubtitleInfo.class);
        var imageResource = new ClassPathResource("images/flags/ar.png");

        when(subtitleInfo.getLanguage()).thenReturn(SubtitleLanguage.NONE);
        when(subtitleInfo.getFlagResource()).thenReturn(imageResource);

        return subtitleInfo;
    }
}