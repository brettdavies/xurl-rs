# Changelog

All notable changes to this project will be documented in this file.

## [0.3.0] - 2026-10-27

### Breaking changes

- Add `user_id` to `OAuth2Token` and `OAuth1Token`, the id of the account the credential belongs to. A struct literal of either names one more field. by @brettdavies in [#328](https://github.com/brettdavies/xurl-rs/pull/328)

  ```rust
  // Before
  let token = OAuth2Token { access_token, refresh_token, expiration_time };

  // After
  let token = OAuth2Token { access_token, refresh_token, expiration_time, user_id: None };
  ```
- Add `account_username` and `display_name` to `OAuth2Token` and `OAuth1Token`, the account's username and display name from the `/2/users/me` answer that stored `user_id`. A struct literal of either names two more fields. by @brettdavies in [#345](https://github.com/brettdavies/xurl-rs/pull/345)

  ```rust
  // Before
  let token = OAuth2Token { access_token, refresh_token, expiration_time, user_id: None };

  // After
  let token = OAuth2Token {
      access_token,
      refresh_token,
      expiration_time,
      user_id: None,
      account_username: None,
      display_name: None,
  };
  ```
- Change `xdk::Error`'s YAML conversion from `serde_yaml::Error` to `serde_saphyr::Error`: the `From` impl takes a `serde_saphyr::Error`, and a token-store YAML failure's `source()` downcasts to `serde_saphyr::Error`. by @brettdavies in [#348](https://github.com/brettdavies/xurl-rs/pull/348)

  ```rust
  // Before
  let err: xdk::Error = serde_yaml::from_str::<Config>(text).unwrap_err().into();
  let yaml = std::error::Error::source(&err).and_then(|s| s.downcast_ref::<serde_yaml::Error>());
  // After
  let err: xdk::Error = serde_saphyr::from_str::<Config>(text).unwrap_err().into();
  let yaml = std::error::Error::source(&err).and_then(|s| s.downcast_ref::<serde_saphyr::Error>());
  ```
- Add `active_follower_count` to `User`, filled by `Client::get_me`. A struct literal of `User` names one more field. by @brettdavies in [#373](https://github.com/brettdavies/xurl-rs/pull/373)

  ```rust
  // Before
  let user = User { id, name, username, ..Default::default() };

  // After: unchanged when built with `..Default::default()`; a literal that names every field adds
  let user = User { id, name, username, active_follower_count: None, ..Default::default() };
  ```

### Added

- Add `store::default_store_path`, `store::locate_store`, and `store::adopt_directory_layout`, with the `store::AUTH_FILE_NAME` and `store::STORE_TARGET` constants. `locate_store` finds a store in either layout and moves nothing; `adopt_directory_layout` moves a single-file store into its directory and reports it as a `tracing` event. by @brettdavies in [#315](https://github.com/brettdavies/xurl-rs/pull/315)
- Add `Call::auth_preflight()` and `Client::auth_preflight(&RequestOptions)`, which report the credential a request would be sent with and send nothing. An expired OAuth2 token is reported, not refreshed. by @brettdavies in [#322](https://github.com/brettdavies/xurl-rs/pull/322)
- Add `AuthPreflight`, their answer: `scheme`, `app`, `username`, and `token_expired`.
- Add `TokenStore::first_oauth2_user_for_app`, the user a request with no username is sent as, with that user's token.
- Add `NextAction::Retry`, `NextAction::FixInput`, and `NextAction::ReportIssue`. `Error::next_action()` returns them for a transport failure or a 5xx, for a 400, 404, 422, an unreadable file, or a refused input, and for a status with no reading of its own, an unreadable body, or a broken invariant. by @brettdavies in [#325](https://github.com/brettdavies/xurl-rs/pull/325)
- Add pointers to `Error::docs_url()`: X's response-code page for a 400, 404, 422, or 5xx, and the issue tracker for a fault of the library.
- Add `AuthPreflight::user_id`, the account id stored beside the credential a request would be sent with. by @brettdavies in [#328](https://github.com/brettdavies/xurl-rs/pull/328)
- Add `TokenStore::set_oauth2_user_id_for_app`, `TokenStore::set_oauth1_user_id_for_app`, and `TokenStore::name_unnamed_oauth2_token_for_app`.
- Add `Client::get_webhooks`, `create_webhook`, `validate_webhook`, `delete_webhook`, and `create_webhook_replay` for the `/2/webhooks` endpoints, with the response types `Webhook`, `WebhookValidation`, and `WebhookReplayJob`. `testing::MockX` answers each. by @brettdavies in [#333](https://github.com/brettdavies/xurl-rs/pull/333)
- Add the offline validators `validate_webhook_url`, `validate_webhook_id`, and `validate_replay_time`, and the constant `WEBHOOK_URL_MAX_CHARS`, which hold the spec's bounds for the webhook endpoints. by @brettdavies in [#334](https://github.com/brettdavies/xurl-rs/pull/334)
- Add `Client::get_account_activity_subscription_count`, `get_account_activity_subscriptions`, `check_account_activity_subscription`, `create_account_activity_subscription`, and `delete_account_activity_subscription`, with the response types `AccountActivitySubscriptionCount`, `AccountActivitySubscriptions`, `AccountActivitySubscription`, and `SubscribedResult`. `testing::MockX` answers each. by @brettdavies in [#336](https://github.com/brettdavies/xurl-rs/pull/336)
- Add the offline validator `validate_user_id`.
- Add `Client::get_webhook_stream_links`, `create_webhook_stream_link`, and `delete_webhook_stream_link`, with the response types `WebhookStreamLink` and `ProvisionedResult`. `testing::MockX` answers each. by @brettdavies in [#337](https://github.com/brettdavies/xurl-rs/pull/337)
- Add the `webhooks` module: `Receiver`, `ReceiverConfig`, `SigningSecrets`, `Event`, `Signature`, and `sign`. A `Receiver` binds an address, answers X's CRC check, and yields each event whose signature verifies. by @brettdavies in [#338](https://github.com/brettdavies/xurl-rs/pull/338)
- Add the refusals `webhook-url-not-https` and `webhook-url-has-port` to `validate_webhook_url`, the two URL rules X documents.
- Add the call shortcuts on `Client`: `get_call`, `get_scheduled_calls`, `schedule_call`, `get_scheduled_call`, `update_scheduled_call`, `cancel_scheduled_call`, and `respond_to_scheduled_call`. They come with the response types `XCall`, `ScheduledCall`, `ScheduledCallInvitee`, `ScheduledCallUpdated`, `ScheduledCallCancelled`, and `ScheduledCallRsvp`, and the request types `NewScheduledCall`, `ScheduledCallChanges`, `Occurrences`, `OccurrenceScope`, and `RsvpAnswer`. The validators are `validate_call_time`, `validate_new_scheduled_call`, `validate_scheduled_call_changes`, and `validate_occurrences`. by @brettdavies in [#340](https://github.com/brettdavies/xurl-rs/pull/340)
- Add the stream-source shortcuts on `Client`: `get_stream_sources`, `create_stream_source`, `get_stream_source`, `rename_stream_source`, and `delete_stream_source`, with `StreamSource`, `StreamSources`, `StreamSourceResult`, `StreamSourceDeleted`, `validate_source_name`, and `validate_source_region`.
- Add `calls.read` and `calls.write` to `get_oauth2_scopes`, the set a sign-in requests when the caller names none.
- Add `Client::get_bookmark_folders`, `Client::create_bookmark_folder`, and `Client::get_bookmark_folder_posts`, with the `BookmarkFolder` and `BookmarkFolderPost` response types and the `validate_bookmark_folder_name` and `validate_bookmark_folder_id` validators. by @brettdavies in [#341](https://github.com/brettdavies/xurl-rs/pull/341)
- Add `TokenStore::set_oauth2_account_for_app` and `TokenStore::set_oauth1_account_for_app`, which store an account's id with its username and display name. by @brettdavies in [#345](https://github.com/brettdavies/xurl-rs/pull/345)
- Add `AuthPreflight::account_username` and `AuthPreflight::display_name`, read offline from the store beside the credential a request would go out under.
- Add `headers` and `received_at` to `webhooks::Event`: the `Content-Type` and signature header the delivery carried, and when it arrived. by @brettdavies in [#353](https://github.com/brettdavies/xurl-rs/pull/353)
- Add `Error::AddressInUse`, which `webhooks::Receiver::bind` returns when another socket holds the address, with kind `address-in-use`, exit code 5, and the OS error as its source. Any other bind failure stays `Error::Io`. by @brettdavies in [#354](https://github.com/brettdavies/xurl-rs/pull/354)
- Add `webhooks::SignedSample`, a sample event signed with the secret a `Receiver` verifies with, and make `OAUTH2_SIGNATURE_HEADER` and `LEGACY_SIGNATURE_HEADER` public. by @brettdavies in [#355](https://github.com/brettdavies/xurl-rs/pull/355)
- Add `testing::deliver_signed_sample`, which posts a signed sample to a bound `Receiver` for an embedder's tests.
- Add `webhooks::UrlTarget`, which forwards a verified event to an HTTP endpoint with X's bytes, `Content-Type`, and signature, an `Idempotency-Key` derived from the bytes, and the caller's headers, following no redirect. A URL or header it could not send is refused before anything is sent. by @brettdavies in [#356](https://github.com/brettdavies/xurl-rs/pull/356)
- Add `webhooks::RetryBudget` with `Attempt`, `AttemptTimedOut`, and `Outcome`: four attempts with 1, 2, and 4 seconds between them, each under an attempt timeout, for a URL or any other target.
- Add `webhooks::ForwardFailure` and `Event::sha256`.
- Add `webhooks::validate_forward_url`, which checks a forward URL offline and names the refusal as `forward-url-invalid`, `forward-url-not-http`, or `forward-url-has-credentials`. `UrlTarget::new` prefixes its URL refusals with the same reason. by @brettdavies in [#357](https://github.com/brettdavies/xurl-rs/pull/357)
- Add `webhooks::CheckpointWriter`, `read_checkpoint`, `replay_window`, and `ReplayWindow`: record how far a webhook receiver got, at most once a minute and atomically, and turn that record into the window X can replay, clamped to X's 24 hours with what is lost named. by @brettdavies in [#359](https://github.com/brettdavies/xurl-rs/pull/359)
- Add `Client::get_my_followers` and `Client::get_my_following` over `/2/users/me/followers` and `/2/users/me/following`, with routes for both in the testing mock. by @brettdavies in [#383](https://github.com/brettdavies/xurl-rs/pull/383)
- Add `Client::hide_reply` over `PUT /2/tweets/{tweet_id}/hidden`, the `HiddenResult` response, and a route for the endpoint in the testing mock. by @brettdavies in [#384](https://github.com/brettdavies/xurl-rs/pull/384)
- Add `Client::get_liking_users`, `Client::get_reposted_by`, and `Client::get_quote_posts`, with routes for all three in the testing mock. by @brettdavies in [#386](https://github.com/brettdavies/xurl-rs/pull/386)
- Add `Client::read_posts` over `GET /2/tweets`, `validate_lookup_ids`, `LOOKUP_IDS_MAX`, and a route for the endpoint in the testing mock. by @brettdavies in [#387](https://github.com/brettdavies/xurl-rs/pull/387)
- Add `Client::get_user_posts` over `GET /2/users/{id}/tweets`, with a route for the endpoint in the testing mock. by @brettdavies in [#388](https://github.com/brettdavies/xurl-rs/pull/388)
- Add `Client::search_users` over `GET /2/users/search`, `validate_user_search_query`, `USER_SEARCH_QUERY_MAX_CHARS`, and a route for the endpoint in the testing mock. by @brettdavies in [#389](https://github.com/brettdavies/xurl-rs/pull/389)
- Add `Client::count_recent_posts` over `GET /2/tweets/counts/recent`, the `PostCount` response, `validate_time_window`, and a route for the endpoint in the testing mock. by @brettdavies in [#390](https://github.com/brettdavies/xurl-rs/pull/390)
- Add `Client::post_analytics` over `GET /2/tweets/analytics`, the `PostAnalytics`, `AnalyticsMetrics`, and `AnalyticsBucket` response types, and a route for the endpoint in the testing mock. by @brettdavies in [#391](https://github.com/brettdavies/xurl-rs/pull/391)
- Add `Client::get_trends` and `Client::get_personalized_trends`, the `Trend` and `PersonalizedTrend` response types, and routes for both endpoints in the testing mock. by @brettdavies in [#392](https://github.com/brettdavies/xurl-rs/pull/392)
- Add `Client::search_news` and `Client::get_news`, the `NewsStory` and `NewsClusterPost` response types, `validate_news_id`, and routes for both endpoints in the testing mock. by @brettdavies in [#393](https://github.com/brettdavies/xurl-rs/pull/393)
- Add `Client::search_communities` and `Client::get_community`, the `Community` response type, `validate_community_id`, and routes for both endpoints in the testing mock. by @brettdavies in [#394](https://github.com/brettdavies/xurl-rs/pull/394)
- Add `Client::create_dm_conversation`, `send_to_conversation`, `get_conversation_events`, `get_dm_events_with`, `get_dm_event`, `delete_dm_event`, `block_dms`, and `unblock_dms`, the `DmBlockResult` response type, `validate_participant_count`, `validate_conversation_id`, `validate_dm_event_id`, `DM_GROUP_PARTICIPANTS_MIN` and `_MAX`, and routes for every endpoint in the testing mock. by @brettdavies in [#395](https://github.com/brettdavies/xurl-rs/pull/395)
- Add the list shortcuts on `Client` (`create_list`, `get_list`, `update_list`, `delete_list`, `get_list_posts`, `get_list_members`, `get_list_followers`, `add_list_member`, `remove_list_member`, `get_owned_lists`, `get_followed_lists`, `get_list_memberships`, `get_pinned_lists`, `follow_list`, `unfollow_list`, `pin_list`, `unpin_list`), the `List`, `ListUpdatedResult`, `ListMemberResult`, and `PinnedResult` response types, `ListDetails`, `validate_list_id`, `validate_new_list`, `validate_list_changes`, `LIST_NAME_MAX_CHARS`, `LIST_DESCRIPTION_MAX_CHARS`, and routes for every list endpoint in the testing mock. by @brettdavies in [#396](https://github.com/brettdavies/xurl-rs/pull/396)
- Add the broadcast shortcuts on `Client` (`list_broadcasts`, `get_broadcast`, `list_scheduled_broadcasts`, `get_scheduled_broadcast`, `create_scheduled_broadcast`, `update_scheduled_broadcast`, `delete_scheduled_broadcast`, `go_live`, `get_broadcast_chat`, `send_broadcast_chat`, `mute_chat_user`, `unmute_chat_user`, `delete_chat_message`), the `Broadcast`, `ScheduledBroadcast`, `BroadcastChatMessage`, `ChatSentResult`, and `ChatMutedResult` response types, the `NewScheduledBroadcast` and `ScheduledBroadcastChanges` inputs, `validate_broadcast_id`, `validate_scheduled_broadcast_id`, `validate_chat_message_id`, `validate_chat_message`, `BROADCAST_CHAT_MAX_CHARS`, and routes for every endpoint in the testing mock. by @brettdavies in [#398](https://github.com/brettdavies/xurl-rs/pull/398)
- Add `Client::create_note`, `evaluate_note`, `delete_note`, `get_notes_written`, and `get_posts_eligible_for_notes`, the `Note`, `NoteInfo`, `NoteCreated`, and `NoteEvaluation` response types, the `NewNote` input, `validate_note_id`, `validate_note_text`, and routes for every endpoint in the testing mock. by @brettdavies in [#399](https://github.com/brettdavies/xurl-rs/pull/399)
- Add `Client::create_article_draft` and `Client::publish_article`, `article_blocks` and `ArticleBlock` for turning plain text into article blocks, the `ArticleDraft` and `ArticlePublished` response types, `validate_article_title`, `validate_article_body`, `validate_article_id`, and routes for both endpoints in the testing mock. by @brettdavies in [#400](https://github.com/brettdavies/xurl-rs/pull/400)
- Add `Client::lookup_users_by_id` and `Client::lookup_users_by_username` over `GET /2/users` and `GET /2/users/by`, `validate_user_lookup`, `validate_username`, `is_user_id_form`, and routes for both endpoints in the testing mock. by @brettdavies in [#401](https://github.com/brettdavies/xurl-rs/pull/401)
- Add `Client::get_media` and `Client::get_media_analytics` over `GET /2/media` and `GET /2/media/analytics`, the `Media`, `MediaAnalytics`, and `MediaAnalyticsBucket` response types, `validate_media_key`, `validate_media_keys`, and routes for both endpoints in the testing mock. by @brettdavies in [#402](https://github.com/brettdavies/xurl-rs/pull/402)
- Add `Client::search_spaces`, `Client::get_spaces`, `Client::get_spaces_by_creator`, and `Client::get_space_posts` over `GET /2/spaces/search`, `GET /2/spaces`, `GET /2/spaces/by/creator_ids`, and `GET /2/spaces/{id}/tweets`; the `Space` response type; `validate_space_id`, `validate_space_ids`, and `user_ids_for_accounts`, which maps a mix of numeric ids and usernames to ids from the answer to a username lookup; and routes for all four endpoints in the testing mock. by @brettdavies in [#403](https://github.com/brettdavies/xurl-rs/pull/403)
- Add `Client::get_stream_rules`, `Client::add_stream_rule`, `Client::remove_stream_rules`, and `Client::get_stream_rule_counts` over `GET` and `POST /2/tweets/search/stream/rules` and `GET /2/tweets/search/stream/rules/counts`; the `StreamRule`, `StreamRuleCounts`, and `StreamRuleAppCount` response types; `validate_rule_id` and `validate_rule_ids`; and routes for the three endpoints in the testing mock. by @brettdavies in [#404](https://github.com/brettdavies/xurl-rs/pull/404)
- Add `validate_accounts`, which checks up to 100 accounts named by any mix of numeric ids and usernames. by @brettdavies in [#407](https://github.com/brettdavies/xurl-rs/pull/407)

### Changed

- Change the default token store to `~/.xurl/auth.yml`. `Config::default_store_path` returns it, and `TokenStore::new`, `TokenStore::with_credentials`, `Auth::new`, and `Client::from_env` open it there, or a single-file store still at `~/.xurl`, without moving anything. by @brettdavies in [#315](https://github.com/brettdavies/xurl-rs/pull/315)
- Change the vendored X API spec to the 2026-10-08 upstream content of 2.169, which adds `created_at` to `ActivityStreamResponse.data` and lets an id in `UpdateRulesDelete.ids` be an integer. `API_SPEC_SHA256` and `API_SPEC_DATE` follow; no typed response or shortcut changes. by @brettdavies in [#327](https://github.com/brettdavies/xurl-rs/pull/327)
- Change `Auth::refresh_oauth2_token` to save the refreshed token where the old one is stored, and to look the account up only when the store lacks its name or id. by @brettdavies in [#328](https://github.com/brettdavies/xurl-rs/pull/328)
- Change the vendored X API spec to upstream 2.172, which adds the calls, stream-source, and XChat endpoints. `API_SPEC_SHA256` and `API_SPEC_DATE` follow. by @brettdavies in [#340](https://github.com/brettdavies/xurl-rs/pull/340)
- Change `TokenStore::set_oauth2_user_id_for_app` and `TokenStore::set_oauth1_user_id_for_app` to drop the stored username and display name when the id changes. by @brettdavies in [#345](https://github.com/brettdavies/xurl-rs/pull/345)
- Change sign-in, refresh, and `Call::send_saving_identity` to store the account's username and display name beside its id.
- Change the token store's YAML to double-quote quoted strings and to quote values a YAML 1.1 reader would misread; stores written in the earlier layout load with every value intact. by @brettdavies in [#348](https://github.com/brettdavies/xurl-rs/pull/348)
- Change the vendored X API spec to the 2026-10-24 upstream content of 2.172, which adds `POST /2/calls`, `POST /2/calls/{id}/join`, `POST /2/calls/{id}/leave`, `GET /2/users/me/followers`, and `GET /2/users/me/following` with their schemas. `API_SPEC_SHA256` and `API_SPEC_DATE` follow, and the auth matrix gains the five operations; no typed response or shortcut changes. by @brettdavies in [#366](https://github.com/brettdavies/xurl-rs/pull/366)
- Change `Client::get_me` to request `active_follower_count`. by @brettdavies in [#373](https://github.com/brettdavies/xurl-rs/pull/373)
- Change `TokenStore::set_oauth2_account_for_app` and `set_oauth1_account_for_app` to also write the names they record onto every `OAuth2` login and `OAuth1` access pair the store holds for the same `user_id`, in every app. Each returns whether the store changed anywhere. A sign-in or refresh that asks `/2/users/me` does the same. A refresh that asks nothing keeps the names the store holds for the slot it writes, so it never writes back names an answer recorded during its grant replaced. by @brettdavies in [#376](https://github.com/brettdavies/xurl-rs/pull/376)

### Deprecated

- Deprecate `POST_BODY_MAX_CHARS`: X sets a post's length limit per account, and nothing in the library checks the constant. by @brettdavies in [#311](https://github.com/brettdavies/xurl-rs/pull/311)

### Fixed

- Fix `Client::get_mentions` and `Client::get_liked_posts` sending a `max_results` outside the 5 to 100 X accepts: the value is brought into that range, as `Client::search_posts` does with 10 to 100. by @brettdavies in [#310](https://github.com/brettdavies/xurl-rs/pull/310)
- Fix `validate_post_body` refusing a body over 280 characters; it returns `empty-body` for an empty one and accepts every other. by @brettdavies in [#311](https://github.com/brettdavies/xurl-rs/pull/311)
- Fix `TokenStore::clear_all`, `clear_oauth1_tokens`, `clear_oauth2_token`, `clear_bearer_token`, and their `_for_app` forms returning `Ok` for a store that failed to load; each returns the token-store error every other write returns. by @brettdavies in [#313](https://github.com/brettdavies/xurl-rs/pull/313)
- Fix scheme auto-detect on a store-backed client ignoring an OAuth2 token stored without a username: a call that names no user is sent with it. by @brettdavies in [#324](https://github.com/brettdavies/xurl-rs/pull/324)
- Fix a refresh without a username saving the new token under the username X answered with, which left the entry it was minted from holding a dead refresh token. by @brettdavies in [#328](https://github.com/brettdavies/xurl-rs/pull/328)
- Fix TLS certificate verification accepting a path through distrusted intermediates, by updating rustls-webpki to 0.103.16 (RUSTSEC-2026-0382). by @brettdavies in [#352](https://github.com/brettdavies/xurl-rs/pull/352)
- Fix RUSTSEC-2026-0385 in the tested dependency set by updating ctutils to 0.4.6. The published manifest is unchanged; an embedder's own lockfile takes the fix with `cargo update -p ctutils`. by @brettdavies in [#369](https://github.com/brettdavies/xurl-rs/pull/369)

### Documentation

- List the `webhooks` module in the README's published surface. by @brettdavies in [#339](https://github.com/brettdavies/xurl-rs/pull/339)
- Add an example to `webhooks::Receiver` that binds and reads events. by @brettdavies in [#353](https://github.com/brettdavies/xurl-rs/pull/353)
- Add the `webhook_forward` example, which receives events, forwards each to an endpoint under `RetryBudget`, and keeps a checkpoint, and name the forwarding and checkpoint items in the README's `webhooks` entry. by @brettdavies in [#361](https://github.com/brettdavies/xurl-rs/pull/361)
- Say in the `testing` module's summary that it is built only with the `testing` feature, in place of the docs.rs feature badge. by @brettdavies in [#364](https://github.com/brettdavies/xurl-rs/pull/364)

**Full Changelog**: [xdk-rs-v0.2.0...xdk-rs-v0.3.0](https://github.com/brettdavies/xurl-rs/compare/xdk-rs-v0.2.0...xdk-rs-v0.3.0)

## [0.2.0] - 2026-10-07

### Breaking changes

- Change `api::execute_media_status` to take the deadline to wait to as `Option<Duration>`, where it took a `bool`. `None` reads the status once. by @brettdavies in [#281](https://github.com/brettdavies/xurl-rs/pull/281)

  ```rust
  // Before
  let status = execute_media_status(id, "", "", true, false, &[], &client).await?;

  // After
  use xdk::api::DEFAULT_PROCESSING_WAIT;
  let status =
      execute_media_status(id, "", "", Some(DEFAULT_PROCESSING_WAIT), false, &[], &client).await?;
  ```
- Add the `reset_at` field to `Error::Api`: the reset a 429's own response named, as seconds since the Unix epoch. A pattern that names every field of the variant gains `..`, and a literal becomes `Error::api`. by @brettdavies in [#282](https://github.com/brettdavies/xurl-rs/pull/282)

  ```rust
  // Before
  match err {
      Error::Api { status, body } => eprintln!("api {status}: {body}"),
      other => eprintln!("{other}"),
  }
  let err = Error::Api { status: 404, body: String::new() };

  // After
  match err {
      Error::Api { status, body, .. } => eprintln!("api {status}: {body}"),
      other => eprintln!("{other}"),
  }
  let err = Error::api(404, "");
  ```
- Change `Error::Http`, `Error::Io`, `Error::Json`, `Error::Auth`, and `Error::TokenStore` from tuple variants holding a message to struct variants holding `message` and `source`, so the lower-level failure is reachable through `std::error::Error::source()`. by @brettdavies in [#284](https://github.com/brettdavies/xurl-rs/pull/284)

  ```rust
  // Before
  match err {
      Error::Auth(message) => eprintln!("sign in again: {message}"),
      Error::Http(_) | Error::Io(_) => eprintln!("check the connection"),
      other => eprintln!("{other}"),
  }
  let err = Error::Io("disk full".to_string());

  // After
  match &err {
      Error::Auth { message, .. } => eprintln!("sign in again: {message}"),
      Error::Http { .. } | Error::Io { .. } => eprintln!("check the connection"),
      other => eprintln!("{other}"),
  }
  let err = Error::io("disk full");
  if let Some(io) = std::error::Error::source(&err).and_then(|s| s.downcast_ref::<std::io::Error>()) {
      eprintln!("caused by {:?}", io.kind());
  }
  ```
- Change `Error::InvalidUrl` from a tuple variant to a struct variant holding `message` and `source`, so a URL that failed to parse carries the `url::ParseError`. by @brettdavies in [#285](https://github.com/brettdavies/xurl-rs/pull/285)

  ```rust
  // Before
  if let Error::InvalidUrl(url) = &err {
      eprintln!("bad URL: {url}");
  }

  // After
  if let Error::InvalidUrl { message, .. } = &err {
      eprintln!("bad URL: {message}");
  }
  ```

### Added

- Add `Auth::with_oauth2_scopes` to narrow the scopes a sign-in requests, `Auth::oauth2_scopes` to read them, `oauth2::narrow_oauth2_scopes` to validate a selection, and `oauth2::OFFLINE_ACCESS`. A sign-in that narrows nothing requests every scope, as before. by @brettdavies in [#277](https://github.com/brettdavies/xurl-rs/pull/277)
- Add `Error::ProcessingTimeout { media_id, waited }` for a wait that reaches its deadline, with `kind()` `processing-timeout` and `next_action()` `NextAction::ResumeWait`. by @brettdavies in [#281](https://github.com/brettdavies/xurl-rs/pull/281)
- Add `MediaUpload::processing_deadline` and `api::DEFAULT_PROCESSING_WAIT` (60 seconds).
- Add `NextAction::WaitAndRetry`, which a 429 that named its reset returns from `next_action()`. by @brettdavies in [#282](https://github.com/brettdavies/xurl-rs/pull/282)
- Add `ClientBuilder::wait_on_rate_limit` and `Config::rate_limit_max_wait`: wait for a 429's reset and send the request once more, when the wait fits the bound. Off by default.
- Add `Error::http`, `Error::io`, and `Error::json` constructors, `Error::with_source` to attach a cause, and the `error::Source` alias for the boxed cause. by @brettdavies in [#284](https://github.com/brettdavies/xurl-rs/pull/284)
- Add `Error::invalid_url` to build the variant from a message. by @brettdavies in [#285](https://github.com/brettdavies/xurl-rs/pull/285)
- Add `MediaUploadOutcome::response()`: FINALIZE's response carrying the final processing state when a wait completed, and FINALIZE's response unchanged otherwise. by @brettdavies in [#296](https://github.com/brettdavies/xurl-rs/pull/296)
- Add `TokenStore::set_default_app_and_user`, which sets the default app and that app's default `OAuth2` user in one write after checking both names. by @brettdavies in [#299](https://github.com/brettdavies/xurl-rs/pull/299)

### Changed

- Change `Client::upload_media` to wait on a video's processing for at most `DEFAULT_PROCESSING_WAIT` unless `processing_deadline` sets another bound. A wait that reaches it reports `Error::ProcessingTimeout` in `MediaUploadOutcome::processing`, with the media id still valid. by @brettdavies in [#281](https://github.com/brettdavies/xurl-rs/pull/281)

### Fixed

- Fix `Auth::refresh_oauth2_token` spending one refresh token twice when two processes, or two `Auth` values on one store, refresh the same expired login concurrently. The refresh runs under `<store>.refresh.lock` and re-reads the stored token first. by @brettdavies in [#279](https://github.com/brettdavies/xurl-rs/pull/279)
- Fix `auth::oauth1::encode` and the OAuth1 signer to percent-encode as RFC 5849 section 3.6 requires: a space is `%20`, `~` stays bare, and `*` is `%2A`. X's published signature example now produces its published `oauth_signature`. by @brettdavies in [#280](https://github.com/brettdavies/xurl-rs/pull/280)
- Fix the processing wait never ending when the status carries no `processing_info`: that status is returned as finished. by @brettdavies in [#281](https://github.com/brettdavies/xurl-rs/pull/281)
- Fix `Client::send_request` and `send_multipart_request` returning `{}` for a success body that is not JSON; the body is returned as a JSON string. by @brettdavies in [#282](https://github.com/brettdavies/xurl-rs/pull/282)
- Fix a non-JSON error body being replaced in `Error::Api`; the body is carried as sent.
- Fix a failed read of a response body returning an empty success; it is `Error::Http`.
- Fix `impl From<url::ParseError> for Error` producing `Error::Http`; it produces `Error::InvalidUrl`, and a raw URL that does not parse is rejected as one before the request is built, on every auth scheme. by @brettdavies in [#285](https://github.com/brettdavies/xurl-rs/pull/285)
- Fix `impl From<serde_yaml::Error> for Error` and the token store's save producing `Error::Json`; they produce `Error::TokenStore`.
- Fix `Client::upload_media` not waiting for a non-video upload whose FINALIZE answer reports processing still under way. `MediaUploadOutcome::processing` is `Some` for such an upload when waiting is on. by @brettdavies in [#297](https://github.com/brettdavies/xurl-rs/pull/297)
- Fix the list shortcuts returning `Error::Json` for a page with no results, which X sends as `meta` with a `result_count` of zero and no `data` key. `ApiResponse<Vec<T>>` decodes it with an empty `data`; a single item with no `data` is still an error. by @brettdavies in [#303](https://github.com/brettdavies/xurl-rs/pull/303)

### Documentation

- Add a Stability section to the README naming the crate's contract: the public Rust API and the behavior its rustdoc documents. by @brettdavies in [#292](https://github.com/brettdavies/xurl-rs/pull/292)
- Replace the README's version-pinned dependency snippets with `cargo add` commands, so the install instructions no longer name a version line that predates the examples beside them. by @brettdavies in [#301](https://github.com/brettdavies/xurl-rs/pull/301)

**Full Changelog**: [xdk-rs-v0.1.4...xdk-rs-v0.2.0](https://github.com/brettdavies/xurl-rs/compare/xdk-rs-v0.1.4...xdk-rs-v0.2.0)

## [0.1.4] - 2026-10-04

### Changed

- Change the vendored X API spec to the 2026-10-02 upstream content of 2.169, which adds an optional `user_id` query parameter to `GET /2/activity/subscriptions`: it narrows the list to subscriptions whose filter names that user, and a user-context token can only name the authenticated user. `API_SPEC_SHA256` and `API_SPEC_DATE` follow; no typed response or shortcut changes. by @brettdavies in [#258](https://github.com/brettdavies/xurl-rs/pull/258)

**Full Changelog**: [xdk-rs-v0.1.3...xdk-rs-v0.1.4](https://github.com/brettdavies/xurl-rs/compare/xdk-rs-v0.1.3...xdk-rs-v0.1.4)

## [0.1.3] - 2026-09-30

### Added

- Add `Client::set_media_alt_text`, `Client::add_media_subtitles`, and `Client::remove_media_subtitles`, with the `MediaMetadataResult` and `MediaSubtitlesResult` response types, the `VideoCategory` enum, and `testing` mock routes for all three endpoints. by @brettdavies in [#252](https://github.com/brettdavies/xurl-rs/pull/252)
- Add the dry-run checks `shortcuts::validate_media_id`, `validate_alt_text`, and `validate_language_code`, and the `ALT_TEXT_MAX_CHARS` constant.

### Changed

- Change the vendored X API spec to 2.169, which adds 23 `/2/chat` operations and removes nothing; `API_SPEC_VERSION`, `API_SPEC_SHA256`, and `API_SPEC_DATE` follow. by @brettdavies in [#252](https://github.com/brettdavies/xurl-rs/pull/252)

### Fixed

- Fix `Client::send_request` and every `Call` dropping the body of a DELETE request; the body and its `Content-Type` now go out as they do for POST, PUT, and PATCH. by @brettdavies in [#252](https://github.com/brettdavies/xurl-rs/pull/252)

**Full Changelog**: [xdk-rs-v0.1.2...xdk-rs-v0.1.3](https://github.com/brettdavies/xurl-rs/compare/xdk-rs-v0.1.2...xdk-rs-v0.1.3)

## [0.1.2] - 2026-09-24

### Changed

- Change the vendored X API spec to the 2026-09-24 upstream content of 2.168, which adds an optional `expires_at` to activity subscriptions: an RFC 3339 expiration on the create request, and the subscription's expiration time in the create, list, and update responses. `API_SPEC_SHA256` and `API_SPEC_DATE` follow; no typed response or shortcut changes. by @brettdavies in [#219](https://github.com/brettdavies/xurl-rs/pull/219)

**Full Changelog**: [xdk-rs-v0.1.1...xdk-rs-v0.1.2](https://github.com/brettdavies/xurl-rs/compare/xdk-rs-v0.1.1...xdk-rs-v0.1.2)

## [0.1.1] - 2026-09-24

### Added

- Add `xdk::api::VOCABULARY_TARGET` (`"xdk::vocabulary"`), the `tracing` target of a `DEBUG` event for each legacy post-vocabulary key a typed response was read under its current name, once per key per response. The event carries `legacy`, `normalized`, `value_type`, `value_len`, and `collision`, and never a value, an id, or a request path. by @brettdavies in [#215](https://github.com/brettdavies/xurl-rs/pull/215)

### Changed

- Change typed responses to read X's legacy post vocabulary under the names the spec uses, at any depth: every `Call` and `deserialize_response` rename a key such as `edit_history_tweet_ids` to `edit_history_post_ids` before deserializing, with the value unchanged, and keep the current spelling when one object carries both. Deserializing the response types directly with serde still accepts the legacy spelling of an aliased field, but fails with serde's `duplicate field` error on an object carrying both spellings; calls through the library never hit it. by @brettdavies in [#214](https://github.com/brettdavies/xurl-rs/pull/214)

  ```text
  // Before: xr post "hi" --output json
  {
    "data": {
      "edit_history_tweet_ids": [
        ""
      ],
      "id": "2101712260468977783",
      "text": "hi"
    }
  }

  // After: xr post "hi" --output json
  {
    "data": {
      "edit_history_post_ids": [
        ""
      ],
      "id": "2101712260468977783",
      "text": "hi"
    }
  }
  ```

### Fixed

- Fix `Post.referenced_posts`, `PostPublicMetrics.repost_count`, and `Includes.posts` reading empty when a body spells them `referenced_tweets`, `retweet_count`, or `tweets`: each field accepts either spelling, as `UserPublicMetrics.post_count` does for `tweet_count`. by @brettdavies in [#213](https://github.com/brettdavies/xurl-rs/pull/213)
- Fix `api::is_streaming_endpoint` returning `false` for seven streaming endpoints the X spec declares. It now answers from the vendored spec's `x-twitter-streaming` marker, so a stream X adds is recognized after the next spec refresh. by @brettdavies in [#220](https://github.com/brettdavies/xurl-rs/pull/220)

### Documentation

- Document `api::VOCABULARY_TARGET` in the crate README's `api` entry, and state on `Includes.posts`, `Post.referenced_posts`, `PostPublicMetrics.repost_count`, and `UserPublicMetrics.post_count` that direct serde deserialization reads the legacy spelling but fails with `duplicate field` on an object carrying both, which the crate's own calls never hit. by @brettdavies in [#216](https://github.com/brettdavies/xurl-rs/pull/216)
- Add a subscriber example for `api::VOCABULARY_TARGET` to the crate docs, and rustdoc search aliases so a search for a legacy name such as `retweet_count` or `edit_history_tweet_ids` lands on the field or target that reads it. by @brettdavies in [#218](https://github.com/brettdavies/xurl-rs/pull/218)
- Give `ApiResponse`, its `data` field, and the four renamed post fields JSON Schema descriptions written for the wire; the Rust-specific detail stays in their rustdoc.
- Document how `xdk-rs` is versioned before 1.0: a breaking change or an MSRV bump moves the middle number, and additions and fixes move the last, so a `^0.y` requirement picks up every compatible release. by @brettdavies in [#228](https://github.com/brettdavies/xurl-rs/pull/228)
- Document on `NextAction` that a newer release can add a member, so a caller treats one it does not recognize as its default branch. by @brettdavies in [#229](https://github.com/brettdavies/xurl-rs/pull/229)
- Document the `0.x` version rule by the number that moves: a breaking change or an MSRV bump moves the middle number (`0.1.x` to `0.2.0`), and an addition or a fix moves the last (`0.1.0` to `0.1.1`), which a `^0.1` requirement picks up. by @brettdavies in [#234](https://github.com/brettdavies/xurl-rs/pull/234)

**Full Changelog**: [xdk-rs-v0.1.0...xdk-rs-v0.1.1](https://github.com/brettdavies/xurl-rs/compare/xdk-rs-v0.1.0...xdk-rs-v0.1.1)

## [0.1.0] - 2026-09-18

### Breaking changes

- Change the error type to `xdk::Error`, re-exported with `xdk::Result` at the crate root and marked `#[non_exhaustive]`, so a match outside the crate needs a wildcard arm. Display strings are lowercase fragments with no prefix and no trailing period, every variant carries an explicit exit code, and the `EnvelopeAlreadyEmitted` variant is gone because it was the binary's control flow rather than anything an embedder can act on. by @brettdavies in [#171](https://github.com/brettdavies/xurl-rs/pull/171)

  ```rust
  // Before
  use xurl::error::{XurlError, exit_code_for_error};

  match client.get_me(&opts) {
      Ok(me) => println!("{}", me.data.username),
      Err(XurlError::Auth(msg)) => {
          eprintln!("{msg}");
          std::process::exit(exit_code_for_error(&XurlError::Auth(msg)));
      }
      Err(err) => return Err(err),
  }

  // After
  use xdk::Error;

  match client.get_me().send().await {
      Ok(me) => println!("{}", me.data.username),
      Err(err @ Error::Auth(_)) => {
          eprintln!("{}: {err}", err.kind());
          std::process::exit(err.exit_code());
      }
      Err(err) => return Err(err),
  }
  ```
- Change the library to async throughout: the client's methods, token exchange and refresh, media upload, and the OAuth2 callback listener are `async fn`s on the caller's runtime, and constructing a client returns `Result`. Shortcuts no longer take a `&CallOptions` by reference at the call site. by @brettdavies in [#172](https://github.com/brettdavies/xurl-rs/pull/172)

  ```rust
  // Before: blocking, options passed by reference
  let posts = client.search_posts("rust", 10, &opts)?;

  // After: async on the caller's runtime
  let posts = client.search_posts("rust", 10).send().await?;
  ```
- Change `stream_request` to return a `StreamLines`, which implements `futures_core::Stream<Item = Result<String>>` and offers `next_line().await`. The caller drives the loop and decides when to stop, and dropping the stream closes the connection.

  ```rust
  // Before
  for line in client.stream_request(&opts)? {
      println!("{}", line?);
  }

  // After
  let mut lines = client.stream_request(&opts).await?;
  while let Some(line) = lines.next_line().await? {
      println!("{line}");
  }
  ```
- Change the browser sign-in to `Client::oauth2_flow(username, cancel, opener)`, which takes a cancellation token and an opener closure and returns the username it signed in. The library performs no terminal output and launches no browser, and it registers no signal handler, so cancellation is the caller's to own.

  ```rust
  // Before
  let username = auth.oauth2_flow(&out, &mut stderr, open::that, None)?;

  // After
  use tokio_util::sync::CancellationToken;

  let username = client
      .oauth2_flow("", CancellationToken::new(), |url| open::that(url))
      .await?;
  ```
- Change `ApiClient` to `Client`, drop the `&CallOptions` parameter from every shortcut, and make `CallOptions` private. A shortcut returns `Call<T>`, finished by `send().await`, which carries the per-call `auth`, `username`, `trace`, `no_auth`, `timeout`, `pagination_token`, and `header` settings that used to live on `CallOptions`. by @brettdavies in [#173](https://github.com/brettdavies/xurl-rs/pull/173)

  ```rust
  // Before
  use xurl::api::{ApiClient, CallOptions};

  let mut opts = CallOptions::default();
  opts.username = Some("brettdavies".to_string());
  let posts = client.search_posts("rust", 10, &opts)?;

  // After
  use xdk::api::Client;

  let posts = client
      .search_posts("rust", 10)
      .username("brettdavies")
      .send()
      .await?;
  ```
- Change `Client::auth()` to an `async fn` returning a `Result`, because a client built from credentials in code holds no token store to hand back.

  ```rust
  // Before: an infallible accessor
  let auth = client.auth();

  // After
  let auth = client.auth().await?;
  ```
- Add `xdk-rs` as the library: the `xurl` library target no longer ships in `xurl-rs`, and every module it exported lives under `xdk::`. The client is async and every shortcut is a builder finished by `.send().await`. `docs/migrating/v4.0.0.md` carries the full rename table. by @brettdavies in [#174](https://github.com/brettdavies/xurl-rs/pull/174)

  ```rust
  // Before: Cargo.toml had xurl-rs = "3"
  use xurl::api::{ApiClient, CallOptions};
  use xurl::auth::Auth;
  use xurl::config::Config;

  let cfg = Config::new();
  let mut client = ApiClient::new(&cfg, Auth::new(&cfg));
  let posts = client.search_posts("rust", 10, &CallOptions::default())?;

  // After: Cargo.toml has xdk-rs = "0.1"
  use xdk::api::Client;
  use xdk::auth::Auth;
  use xdk::config::Config;

  let cfg = Config::new();
  let client = Client::new(&cfg, Auth::new(&cfg))?;
  let posts = client.search_posts("rust", 10).send().await?;
  ```
- Change the published surface to what the documentation names. The OAuth1 signing primitives, the OAuth2 callback listener, the pending sign-in state, the token-store snapshot, and the raw-URL media helpers are no longer part of the documented API, and `Call::auth_wire` is `Call::auth_wire_unchecked` because it bypasses the auth-method check. by @brettdavies in [#175](https://github.com/brettdavies/xurl-rs/pull/175)

  ```rust
  // Before
  let call = client.search_posts("rust", 10).auth_wire("oauth2");

  // After
  let call = client.search_posts("rust", 10).auth_wire_unchecked("oauth2");
  ```
- Change `Error::AuthMethodMismatch` to carry a boxed `AuthMismatch` struct, which brings `xdk::Error` to 48 bytes so a function returning `xdk::Result` no longer trips clippy's `result_large_err`. by @brettdavies in [#178](https://github.com/brettdavies/xurl-rs/pull/178)

  ```rust
  // Before
  return Err(Error::AuthMethodMismatch {
      requested: "oauth2".into(),
      required: "oauth1".into(),
  });

  // After
  return Err(Error::from(AuthMismatch {
      requested: "oauth2".into(),
      required: "oauth1".into(),
  }));
  ```
- Remove the unread `pagination_token` field from `RequestOptions` and `execute_media_upload` from the documented API. A list shortcut's cursor is `Call::pagination_token`, and `Client::upload_media` is the embedder path.

  ```rust
  // Before
  let mut opts = RequestOptions::default();
  opts.pagination_token = Some(cursor);

  // After
  let page = client.get_bookmarks().pagination_token(cursor).send().await?;
  ```
- Change `Client::send_dm` to return `Call<ApiResponse<DmSentResult>>` instead of `Call<ApiResponse<DmEvent>>`. The API's reply never carried the fields `DmEvent` declared, so the call failed deserialization; read `dm_event_id` where you read `id`, and `dm_conversation_id` for the conversation. by @brettdavies in [#194](https://github.com/brettdavies/xurl-rs/pull/194)

  ```rust
  // Before
  let sent = client.send_dm(participant_id, "hello").send().await?;
  println!("{}", sent.data.id);

  // After
  let sent = client.send_dm(participant_id, "hello").send().await?;
  println!("{} in {}", sent.data.dm_event_id, sent.data.dm_conversation_id);
  ```
- Change `Error::kind()` to name an API refusal by its status: 403 reports `forbidden`, 400 and 422 report `invalid-request`, 5xx reports `server-error`, and any other status reports `api-error`. `network-error` now names only a request that got no answer, and `Error::exit_code()` returns 5 for a transport failure instead of a code read from digits in the request URL. by @brettdavies in [#198](https://github.com/brettdavies/xurl-rs/pull/198)

  ```rust
  // Before: every status but 401, 404 and 429 arrived as a transport failure
  match err.kind() {
      "network-error" => retry_after_backoff(),
      other => report(other),
  }

  // After: a refusal names itself, and network-error means no answer came back
  match err.kind() {
      "forbidden" => check_app_permissions(),
      "invalid-request" => fix_request(),
      "server-error" | "api-error" => retry_after_backoff(),
      "network-error" => retry_after_backoff(),
      other => report(other),
  }
  ```

### Added

- Add `Error::docs_url()`, which names the documentation page for an enrollment refusal, a credential failure, or a rate limit. by @brettdavies in [#171](https://github.com/brettdavies/xurl-rs/pull/171)
- Add `Client::builder()`, so a consumer can build a client from a bearer token, an OAuth2 token pair, or OAuth1 credentials held in code, with no token store and no environment variable. by @brettdavies in [#173](https://github.com/brettdavies/xurl-rs/pull/173)

  ```rust
  let client = Client::builder()
      .bearer(std::env::var("BEARER")?)
      .build()?;
  ```
- Add the `OnTokenRefreshed` hook: the client hands every rotated OAuth2 pair to it, `TokenStore` implements it for its default app, and `TokenStore::refresh_hook_for(app)` scopes it to a named app.
- Add `Client::last_rate_limit()` and the `RateLimit` type, reporting the `x-rate-limit-*` window from the most recent response. by @brettdavies in [#175](https://github.com/brettdavies/xurl-rs/pull/175)
- Add the `rustls` (default) and `native-tls` Cargo features, which select the TLS backend. A build with neither fails at compile time with the fix named rather than at the first request.
- Add a doctested README as the crate's docs.rs landing page, opening with a complete bearer-token program and covering authentication, errors, rate limits, testing, and the TLS features. by @brettdavies in [#176](https://github.com/brettdavies/xurl-rs/pull/176)
- Add `examples/`: read a post, search, authenticate with the refresh hook, post with media, stream, and a credential-free `offline_search`.
- Add the `testing` feature and `xdk::testing::MockX`, an in-process mock of the API seeded from the crate's own fixtures, so a consumer's tests need no credentials and spend nothing.
- Add `Client::upload_media(path)`, a `MediaUpload` builder that infers the media type and category from the file, takes the same per-call `auth`, `username`, `trace`, and `header` settings a `Call` does, awaits a video's processing, and returns a `MediaUploadOutcome` whose `media_id()` feeds `create_post`. by @brettdavies in [#178](https://github.com/brettdavies/xurl-rs/pull/178)
- Add `Client::get_chat_moderators`, `add_chat_moderator`, and `remove_chat_moderator`, with a `ChatModeratorsResult` response type and a `chat_moderators` fixture in the `testing` mock. by @brettdavies in [#182](https://github.com/brettdavies/xurl-rs/pull/182)
- Add routes for every declared shortcut endpoint to the `testing` mock (`MockX`): the user-graph reads, the undo verbs, the media upload phases, and usage credits. by @brettdavies in [#191](https://github.com/brettdavies/xurl-rs/pull/191)
- Add `crates/xdk/CHANGELOG.md`, generated on the release branch like the binary's. by @brettdavies in [#202](https://github.com/brettdavies/xurl-rs/pull/202)

### Changed

- Change the client to a cheap-`Clone`, `&self` handle that shares one HTTP client and one credential lock, so a token refresh cannot be lost to a dropped request or duplicated by a concurrent one. by @brettdavies in [#172](https://github.com/brettdavies/xurl-rs/pull/172)
- Change `ClientBuilder`, `Call`, `Auth`, and `TokenStore` to implement `Debug` with every secret redacted. by @brettdavies in [#176](https://github.com/brettdavies/xurl-rs/pull/176)
- Change `Client::from_env()` to build over the `~/.xurl` store without `CLIENT_ID` exported; the store's default app supplies the client id. by @brettdavies in [#178](https://github.com/brettdavies/xurl-rs/pull/178)
- Change the OAuth2 scopes requested at enrollment to include `broadcast.read` and `broadcast.write`; a token issued before this release must be re-enrolled before the broadcast shortcuts work. by @brettdavies in [#182](https://github.com/brettdavies/xurl-rs/pull/182)

### Fixed

- Fix credential writes made during a legacy-store migration or a twurlrc import bypassing the store lock, and make every store write durable across a crash, unique per writer, and safe on a symlinked store path. by @brettdavies in [#171](https://github.com/brettdavies/xurl-rs/pull/171)
- Fix a token exchange, refresh, or username lookup hanging forever when the HTTP client builder failed: the client is built once, and a build failure is an error rather than a fallback with no timeout. by @brettdavies in [#172](https://github.com/brettdavies/xurl-rs/pull/172)
- Fix a token refresh whose response omits `refresh_token` dropping the existing one; the stored refresh token is kept. by @brettdavies in [#177](https://github.com/brettdavies/xurl-rs/pull/177)
- Fix the headless OAuth2 sign-in writing its pending state through a symlink at the pending path; it refuses instead.
- Fix `Debug` output of the token store's app and token types to redact every credential.
- Fix warnings raised while the token store loads, such as a rejected `REDIRECT_URI` or a legacy-store migration, reaching stderr again.
- Fix `Config` and `EnvOverrides` printing the client secret and bearer token under `Debug`. by @brettdavies in [#178](https://github.com/brettdavies/xurl-rs/pull/178)
- Fix credential-store writes so a crash or a concurrent writer can no longer leave the store truncated or drop a rotated refresh token: saves are atomic, `0600` from creation, and serialized by an OS file lock beside the store. by @brettdavies in [#180](https://github.com/brettdavies/xurl-rs/pull/180)
