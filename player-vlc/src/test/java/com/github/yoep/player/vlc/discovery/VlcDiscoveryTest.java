package com.github.yoep.player.vlc.discovery;

import com.github.yoep.player.vlc.VlcPlayer;
import com.github.yoep.player.vlc.services.VlcPlayerService;
import com.github.yoep.popcorn.backend.adapters.player.PlayerManagerService;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.extension.ExtendWith;
import org.junit.jupiter.api.io.TempDir;
import org.mockito.InjectMocks;
import org.mockito.Mock;
import org.mockito.junit.jupiter.MockitoExtension;

import java.io.File;
import java.io.IOException;
import java.util.function.Supplier;

import static org.junit.jupiter.api.Assertions.assertEquals;
import static org.mockito.Mockito.*;

@ExtendWith(MockitoExtension.class)
class VlcDiscoveryTest {
    @Mock
    private PlayerManagerService playerManagerService;
    @Mock
    private VlcPlayerService vlcPlayerService;
    @InjectMocks
    private VlcDiscovery discovery;
    @TempDir
    File workingDir;

    @Test
    void testDiscover_whenSystemPathContainsVlcExecutable_shouldRegisterVlcPlayer() throws IOException {
        var executable = new File(workingDir.getAbsolutePath() + File.separator + VlcDiscovery.FILENAME);
        var expectedResult = new VlcPlayer(vlcPlayerService);
        executable.createNewFile();
        discovery.environmentPathSupplier = () -> workingDir.getAbsolutePath();

        discovery.init();

        verify(playerManagerService, timeout(250)).register(expectedResult);
        assertEquals(workingDir.getAbsolutePath(), discovery.discoveredVlcPath);
    }

    @Test
    void testDiscover_whenSystemPathDoesNotContainVlcExecutable_shouldNotRegisterVlcPlayer() {
        var supplier = (Supplier<String>) mock(Supplier.class);
        when(supplier.get()).thenReturn(workingDir.getAbsolutePath());
        discovery.environmentPathSupplier = supplier;

        discovery.init();
        verify(supplier, timeout(250)).get();

        verify(playerManagerService, times(0)).register(isA(VlcPlayer.class));
    }
}