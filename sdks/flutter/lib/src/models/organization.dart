class NucleusOrganization {
  final String id, name, slug;
  final String? logoUrl;
  NucleusOrganization({required this.id, required this.name, required this.slug, this.logoUrl});
  factory NucleusOrganization.fromJson(Map<String, dynamic> json) => NucleusOrganization(
    id: json['id'], name: json['name'], slug: json['slug'], logoUrl: json['logo_url']);
}
