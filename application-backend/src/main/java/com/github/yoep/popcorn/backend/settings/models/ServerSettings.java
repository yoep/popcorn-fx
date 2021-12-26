package com.github.yoep.popcorn.backend.settings.models;

import lombok.*;
import org.springframework.lang.Nullable;

import java.net.URI;
import java.util.Objects;
import java.util.Optional;

@EqualsAndHashCode(callSuper = false)
@Data
@Builder
@NoArgsConstructor
@AllArgsConstructor
public class ServerSettings extends AbstractSettings {
    public static final String API_SERVER_PROPERTY = "apiServer";

    //region Properties

    /**
     * The custom api server to use for retrieving {@link com.github.yoep.popcorn.ui.media.providers.models.Media} information.
     * This server will be added on the top of the api list if configured.
     */
    @Nullable
    private URI apiServer;

    //endregion

    //region Getters & Setters

    public Optional<URI> getApiServer() {
        return Optional.ofNullable(apiServer);
    }

    public void setApiServer(URI apiServer) {
        if (Objects.equals(this.apiServer, apiServer))
            return;

        var oldValue = this.apiServer;
        this.apiServer = apiServer;
        changes.firePropertyChange(API_SERVER_PROPERTY, oldValue, apiServer);
    }

    //endregion
}
