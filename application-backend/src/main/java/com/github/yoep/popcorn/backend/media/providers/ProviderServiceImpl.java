package com.github.yoep.popcorn.backend.media.providers;

import com.github.yoep.popcorn.backend.lib.FxChannel;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.*;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.*;
import com.github.yoep.popcorn.backend.lib.ipc.protobuf.Media.Error;
import com.github.yoep.popcorn.backend.media.Media;
import com.github.yoep.popcorn.backend.media.MediaException;
import com.github.yoep.popcorn.backend.media.MediaHelper;
import lombok.extern.slf4j.Slf4j;

import java.util.List;
import java.util.Objects;
import java.util.concurrent.CompletableFuture;
import java.util.stream.Collectors;

@Slf4j
public class ProviderServiceImpl implements ProviderService<com.github.yoep.popcorn.backend.media.Media> {
    private final FxChannel fxChannel;

    public ProviderServiceImpl(FxChannel fxChannel) {
        Objects.requireNonNull(fxChannel, "fxChannel cannot be null");
        this.fxChannel = fxChannel;
    }

    @Override
    public CompletableFuture<List<com.github.yoep.popcorn.backend.media.Media>> getPage(Category category, Genre genre, SortBy sortBy, int page) {
        return doInternalPageRetrieval(category, genre, sortBy, "", page);
    }

    @Override
    public CompletableFuture<List<com.github.yoep.popcorn.backend.media.Media>> getPage(Category category, Genre genre, SortBy sortBy, int page, String keywords) {
        return doInternalPageRetrieval(category, genre, sortBy, keywords, page);
    }

    @Override
    public CompletableFuture<com.github.yoep.popcorn.backend.media.Media> retrieveDetails(Media media) {
        return fxChannel.send(
                GetMediaDetailsRequest.newBuilder()
                        .setItem(MediaHelper.getItem(media))
                        .build(),
                GetMediaDetailsResponse.parser()
        ).thenApply(response -> {
            if (response.getResult() == Response.Result.OK) {
                return MediaHelper.getMedia(response.getItem());
            } else {
                var error = response.getError();
                if (error.getType() == Error.Type.NO_AVAILABLE_PROVIDERS) {
                    throw new MediaRetrievalException("no providers are available");
                }

                throw new MediaException("failed to retrieve media details");
            }
        });
    }

    @Override
    public void resetApiAvailability(Category category) {
        fxChannel.send(ResetProviderApiRequest.newBuilder()
                .setCategory(category)
                .build());
    }

    private CompletableFuture<List<Media>> doInternalPageRetrieval(Category category, Genre genre, SortBy sortBy, String keywords, int page) {
        return fxChannel.send(GetMediaItemsRequest.newBuilder()
                        .setCategory(category)
                        .setGenre(genre)
                        .setSortBy(sortBy)
                        .setKeywords(keywords)
                        .setPage(page)
                        .build(), GetMediaItemsResponse.parser())
                .thenApply(response -> {
                    if (response.getResult() == Response.Result.OK) {
                        return response.getItemsList().stream()
                                .map(MediaHelper::getMedia)
                                .collect(Collectors.toList());
                    } else {
                        var error = response.getError();
                        log.error("Failed to retrieve favorites, {}", error.getType());
                        throw new MediaException("failed to retrieve favorites");
                    }
                });
    }
}
