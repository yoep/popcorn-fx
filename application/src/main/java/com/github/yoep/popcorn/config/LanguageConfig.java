package com.github.yoep.popcorn.config;

import org.springframework.context.annotation.Bean;
import org.springframework.context.annotation.Configuration;
import org.springframework.context.support.ResourceBundleMessageSource;

@Configuration
public class LanguageConfig {
    private static final String DIRECTORY = "lang/";

    @Bean
    public ResourceBundleMessageSource messageSource() {
        ResourceBundleMessageSource messageSource = new ResourceBundleMessageSource();
        messageSource.setBasenames(DIRECTORY + "main", DIRECTORY + "genres", DIRECTORY + "sort-by", DIRECTORY + "languages");
        return messageSource;
    }
}
