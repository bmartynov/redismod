CARGO := cargo

.PHONY: fix
fix:
	@echo ":: Running cargo fix..." && \
	$(CARGO) fix --lib

	@echo ":: Running cargo fmt..." && \
	$(CARGO) fmt

.PHONY: fix-dirty
fix-dirty:
	@echo ":: Running cargo fix..." && \
	$(CARGO) fix --lib --allow-dirty --allow-staged

	@echo ":: Running cargo fmt..." && \
	$(CARGO) fmt