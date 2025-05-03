package com.github.yoep.popcorn.ui.view.controllers.desktop.sections;

import com.github.yoep.popcorn.backend.events.EventPublisher;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.MagnetInfo;
import com.github.yoep.popcorn.backend.loader.LoaderService;
import com.github.yoep.popcorn.backend.utils.LocaleText;
import com.github.yoep.popcorn.ui.events.ShowTorrentCollectionEvent;
import com.github.yoep.popcorn.backend.torrent.TorrentCollectionService;
import com.github.yoep.popcorn.ui.torrent.controls.TorrentCollection;
import javafx.scene.layout.Pane;
import org.junit.jupiter.api.BeforeEach;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.Spy;
import org.mockito.junit.jupiter.MockitoExtension;
import org.testfx.framework.junit5.ApplicationExtension;
import org.testfx.util.WaitForAsyncUtils;

import java.net.URL;
import java.util.Collections;
import java.util.ResourceBundle;
import java.util.concurrent.CompletableFuture;

import static org.mockito.Mockito.*;

@ExtendWith({MockitoExtension.class, ApplicationExtension.class})
class TorrentCollectionSectionControllerTest {
    @Spy
    private EventPublisher eventPublisher = new EventPublisher(false);
    @Mock
    private TorrentCollectionService torrentCollectionService;
    @Mock
    private LocaleText localeText;
    @Mock
    private LoaderService loaderService;
    @Mock
    private URL url;
    @Mock
    private ResourceBundle resourceBundle;
    @InjectMocks
    private TorrentCollectionSectionController controller;

    @BeforeEach
    void setUp() {
        controller.fileShadow = new Pane();
        controller.collection = new TorrentCollection();
    }

    @Test
    void testOnItemClicked() {
        var uri = "magnet://example-url";
        var torrent = MagnetInfo.newBuilder()
                .setMagnetUri(uri)
                .build();
        when(torrentCollectionService.getStoredTorrents()).thenReturn(CompletableFuture.completedFuture(Collections.singletonList(torrent)));
        controller.initialize(url, resourceBundle);

        eventPublisher.publish(new ShowTorrentCollectionEvent(this));
        WaitForAsyncUtils.waitForFxEvents();
        controller.collection.getTorrentClickedConsumer().accept(torrent);

        verify(loaderService).load(uri);
    }
}