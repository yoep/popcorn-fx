package com.github.yoep.popcorn.controllers;

import org.springframework.scheduling.annotation.Async;

import java.util.concurrent.CompletableFuture;

public interface LoginController {
    /**
     * Start the authorization process for the given url.
     *
     * @param url                 The url for the authorization.
     * @param expectedRedirectUrl The expected redirect url that should contain the access token.
     * @return Returns the redirect url.
     */
    @Async
    CompletableFuture<String> startAuthorization(String url, String expectedRedirectUrl);
}
