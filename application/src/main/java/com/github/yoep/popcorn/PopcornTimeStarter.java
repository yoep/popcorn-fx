package com.github.yoep.popcorn;

import com.github.yoep.player.chromecast.ChromecastPlayer;
import com.github.yoep.player.chromecast.discovery.DiscoveryService;
import com.github.yoep.player.chromecast.discovery.FfmpegDiscovery;
import com.github.yoep.player.chromecast.services.ChromecastService;
import com.github.yoep.player.chromecast.services.MetaDataService;
import com.github.yoep.player.popcorn.controllers.components.*;
import com.github.yoep.player.popcorn.player.EmbeddablePopcornPlayer;
import com.github.yoep.player.popcorn.player.PopcornPlayer;
import com.github.yoep.popcorn.backend.lib.FxLibInstance;
import com.github.yoep.popcorn.backend.lib.PopcornFxInstance;
import com.github.yoep.popcorn.backend.settings.ApplicationConfig;
import com.github.yoep.popcorn.ui.ApplicationArgs;
import com.github.yoep.popcorn.ui.IoC;
import com.github.yoep.popcorn.ui.PopcornTimeApplication;
import com.github.yoep.popcorn.ui.PopcornTimePreloader;
import com.github.yoep.torrent.frostwire.*;
import com.github.yoep.video.javafx.VideoPlayerFX;
import com.github.yoep.video.vlc.VideoPlayerVlc;
import com.github.yoep.video.youtube.VideoPlayerYoutube;
import com.github.yoep.vlc.discovery.LinuxNativeDiscoveryStrategy;
import com.github.yoep.vlc.discovery.OsxNativeDiscoveryStrategy;
import com.github.yoep.vlc.discovery.WindowsNativeDiscoveryStrategy;
import com.sun.jna.Memory;
import com.sun.jna.Native;
import com.sun.jna.Pointer;
import javafx.application.Application;
import javafx.application.Preloader;
import org.apache.http.impl.client.DefaultRedirectStrategy;
import org.apache.http.impl.client.HttpClientBuilder;
import uk.co.caprica.vlcj.factory.MediaPlayerFactory;
import uk.co.caprica.vlcj.factory.discovery.NativeDiscovery;
import uk.co.caprica.vlcj.factory.discovery.strategy.NativeDiscoveryStrategy;

import java.nio.charset.StandardCharsets;
import java.util.Arrays;
import java.util.Optional;

public class PopcornTimeStarter {
    public static void main(String[] args) {
        System.setProperty("jna.encoding", StandardCharsets.UTF_8.name());
        var ioc = PopcornTimeApplication.getIOC();
        ioc.registerInstance(new ApplicationArgs(args));

        var libArgs = createLibraryArguments(args);
        var fxLib = ioc.registerInstance(FxLibInstance.INSTANCE.get());
        var popcornFx = ioc.registerInstance(fxLib.new_popcorn_fx(libArgs.ptr, libArgs.length));
        PopcornFxInstance.INSTANCE.set(popcornFx);

        PopcornTimeApplication.getON_INIT().set(PopcornTimeStarter::onInit);
        launch(PopcornTimeApplication.class, PopcornTimePreloader.class, args);
    }

    static void launch(Class<? extends Application> appClass, Class<? extends Preloader> preloaderClass, String... args) {
        System.setProperty("javafx.preloader", preloaderClass.getName());
        Application.launch(appClass, args);
    }

    static Args createLibraryArguments(String[] args) {
        var length = args.length + 1;
        var libArgs = new String[length];
        libArgs[0] = "popcorn-fx";
        System.arraycopy(args, 0, libArgs, 1, args.length);
        var pointer = new Memory((long) length * Native.POINTER_SIZE);
        var stringPointers = new Pointer[length];

        for (int i = 0; i < length; i++) {
            stringPointers[i] = new Memory(libArgs[i].length() + 1);
            stringPointers[i].setString(0, libArgs[i]);
            pointer.setPointer((long) i * Native.POINTER_SIZE, stringPointers[i]);
        }

        return new Args(pointer, length);
    }

    private static void onInit(IoC ioC) {
        var client = HttpClientBuilder.create()
                .setRedirectStrategy(new DefaultRedirectStrategy())
                .build();
        var discoveryStrategies = Arrays.<NativeDiscoveryStrategy>asList(
                new LinuxNativeDiscoveryStrategy(),
                new OsxNativeDiscoveryStrategy(),
                new WindowsNativeDiscoveryStrategy()
        );

        // torrent services
        ioC.register(TorrentServiceImpl.class);
        ioC.register(TorrentSessionManagerImpl.class);
        ioC.register(TorrentSettingsServiceImpl.class);
        ioC.registerInstance(new TorrentResolverService(ioC.getInstance(TorrentSessionManager.class), client));

        // popcorn fx player
        ioC.registerInstance(new NativeDiscovery(discoveryStrategies.toArray(NativeDiscoveryStrategy[]::new)));
        ioC.register(MediaPlayerFactory.class);
        ioC.register(VideoPlayerYoutube.class);
        ioC.register(VideoPlayerVlc.class);
        ioC.register(VideoPlayerFX.class);
        ioC.register(EmbeddablePopcornPlayer.class);
        ioC.register(PopcornPlayer.class);
        ioC.register(PlayerHeaderComponent.class);
        ioC.register(PlayerSubtitleComponent.class);
        ioC.register(PlayerPlaylistComponent.class);

        // chromecast
        ioC.register(ChromecastPlayer.class);
        ioC.register(ChromecastService.class);
        ioC.register(MetaDataService.class);
        ioC.register(DiscoveryService.class);
        Optional.ofNullable(FfmpegDiscovery.discoverProbe()).ifPresent(ioC::registerInstance);

        if (isDesktopMode(ioC)) {
            onInitDesktop(ioC);
        } else {
            onInitTv(ioC);
        }
    }

    private static void onInitDesktop(IoC ioC) {
        // popcorn fx player
        ioC.register(DesktopHeaderActionsComponent.class);
        ioC.register(DesktopPlayerControlsComponent.class);
    }

    private static void onInitTv(IoC ioC) {
        // popcorn fx player
        ioC.register(TvPlayerControlsComponent.class);
    }

    private static boolean isDesktopMode(IoC ioC) {
        return !ioC.getInstance(ApplicationConfig.class).isTvMode();
    }

    private record Args(Pointer ptr, int length) {
    }
}
