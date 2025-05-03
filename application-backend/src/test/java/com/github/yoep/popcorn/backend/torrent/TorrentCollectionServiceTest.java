package com.github.yoep.popcorn.backend.torrent;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.google.protobuf.Parser;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.util.concurrent.CompletableFuture;
import java.util.concurrent.atomic.AtomicReference;

import static org.junit.jupiter.api.Assertions.*;
import static org.mockito.ArgumentMatchers.isA;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class TorrentCollectionServiceTest {
    @Mock
    private FxChannel fxChannel;
    @InjectMocks
    private TorrentCollectionService service;

    @Test
    void testIsStored() {
        var uri = "magnet:?SomeTorrent";
        var request = new AtomicReference<IsMagnetUriStoredRequest>();
        when(fxChannel.send(isA(IsMagnetUriStoredRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, IsMagnetUriStoredRequest.class));
            return CompletableFuture.completedFuture(IsMagnetUriStoredResponse.newBuilder()
                    .setIsStored(true)
                    .build());
        });

        var result = service.isStored(uri).resultNow();

        verify(fxChannel).send(isA(IsMagnetUriStoredRequest.class), isA(Parser.class));
        assertTrue(result, "expected the uri to have been stored");
        assertEquals(uri, request.get().getMagnetUri());
    }

    @Test
    void testGetStoredTorrents() {
        var magnet = MagnetInfo.newBuilder()
                .setMagnetUri("magnet:?LoremIpsum")
                .build();
        var request = new AtomicReference<GetTorrentCollectionRequest>();
        when(fxChannel.send(isA(GetTorrentCollectionRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, GetTorrentCollectionRequest.class));
            return CompletableFuture.completedFuture(GetTorrentCollectionResponse.newBuilder()
                    .addTorrents(magnet)
                    .build());
        });

        var result = service.getStoredTorrents().resultNow();

        verify(fxChannel).send(isA(GetTorrentCollectionRequest.class), isA(Parser.class));
        assertNotNull(request, "expected a request to have been sent");
        assertEquals(magnet, result.getFirst());
    }

    @Test
    void testAddTorrent() {
        var info = Torrent.Info.newBuilder()
                .setUri("magnet:?MyTorrentUri")
                .build();
        var request = new AtomicReference<AddTorrentCollectionRequest>();
        when(fxChannel.send(isA(AddTorrentCollectionRequest.class), isA(Parser.class))).thenAnswer(invocations -> {
            request.set(invocations.getArgument(0, AddTorrentCollectionRequest.class));
            return CompletableFuture.completedFuture(AddTorrentCollectionResponse.newBuilder()
                    .setResult(Response.Result.OK)
                    .build());
        });

        service.addTorrent(info);

        verify(fxChannel).send(isA(AddTorrentCollectionRequest.class), isA(Parser.class));
        assertEquals(FxChannel.typeFrom(Torrent.Info.class), request.get().getType());
        assertEquals(info, request.get().getTorrentInfo());
    }

    @Test
    void testRemoveTorrent() {
        var uri = "magnet:?TorrentToRemoveUri";
        var request = new AtomicReference<RemoveTorrentCollectionRequest>();
        doAnswer(invocations -> {
            request.set(invocations.getArgument(0, RemoveTorrentCollectionRequest.class));
            return null;
        }).when(fxChannel).send(isA(RemoveTorrentCollectionRequest.class));

        service.removeTorrent(uri);

        verify(fxChannel).send(isA(RemoveTorrentCollectionRequest.class));
        assertEquals(FxChannel.typeFrom(MagnetInfo.class), request.get().getType());
        assertEquals(uri, request.get().getMagnetInfo().getMagnetUri());
    }
}